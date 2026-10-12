//! The HTTP side of the Streamable HTTP transport: POST `/mcp` on the
//! loopback address only, one JSON answer per message, and no event
//! stream (the server sends no notifications). Requests from web pages are
//! refused by their `Origin`.

use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use tiny_http::{Header, Method, Response, Server, StatusCode};

use crate::bridge::Bridge;
use crate::protocol;
use crate::status::{ServerState, Status};

/// The port used unless the user picks another.
pub const DEFAULT_PORT: u16 = 7373;
/// The endpoint's path.
pub const PATH: &str = "/mcp";
/// Largest request body (a 50 MB image in base64, and its JSON).
const MAX_BODY: u64 = 72 * 1024 * 1024;

/// The endpoint's URL for `port`.
pub fn url(port: u16) -> String {
    format!("http://127.0.0.1:{port}{PATH}")
}

/// A running server; stopped when dropped.
pub struct ServerHandle {
    server: Option<Arc<Server>>,
    thread: Option<JoinHandle<()>>,
    pub port: u16,
}

/// Why the server didn't start.
#[derive(Debug)]
pub enum StartError {
    PortInUse,
    Other(String),
}

/// Starts listening on `127.0.0.1:port` and answering on a thread of its
/// own; `status` follows its state.
pub fn start(
    port: u16,
    bridge: Bridge,
    status: Arc<Mutex<Status>>,
) -> Result<ServerHandle, StartError> {
    let set = |state: ServerState| {
        if let Ok(mut s) = status.lock() {
            s.reset();
            s.port = port;
            s.state = state;
        }
    };
    let server = match Server::http(("127.0.0.1", port)) {
        Ok(server) => Arc::new(server),
        Err(err) => {
            let in_use = err
                .downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::AddrInUse);
            let error = if in_use {
                set(ServerState::PortInUse);
                StartError::PortInUse
            } else {
                set(ServerState::Failed(err.to_string()));
                StartError::Other(err.to_string())
            };
            tracing::warn!(port, %err, "MCP server not started");
            return Err(error);
        }
    };
    let worker = server.clone();
    let thread_status = status.clone();
    let spawned = std::thread::Builder::new()
        .name("mcp-server".into())
        .spawn(move || {
            for request in worker.incoming_requests() {
                serve(request, port, &bridge, &thread_status);
            }
        });
    match spawned {
        Ok(thread) => {
            set(ServerState::Listening);
            tracing::info!(port, "MCP server listening");
            Ok(ServerHandle {
                server: Some(server),
                thread: Some(thread),
                port,
            })
        }
        Err(err) => {
            set(ServerState::Failed(err.to_string()));
            Err(StartError::Other(err.to_string()))
        }
    }
}

impl ServerHandle {
    /// Stops listening and waits for the thread (the app's end of the
    /// bridge should be dropped first, so a waiting call ends at once).
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        let Some(server) = self.server.take() else {
            return;
        };
        server.unblock();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        // The last reference: tiny_http wakes its accept thread, which then
        // closes the listening socket on its own. Wait for it, so the port is
        // free (and refuses connections) once this returns.
        drop(server);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while std::net::TcpListener::bind(("127.0.0.1", self.port)).is_err()
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        tracing::info!(port = self.port, "MCP server stopped");
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Whether a request's `Origin` (when it has one) is this server's page.
fn origin_allowed(origin: Option<&str>, port: u16) -> bool {
    match origin {
        None => true,
        Some(o) => {
            let o = o.trim().trim_end_matches('/');
            o == format!("http://127.0.0.1:{port}") || o == format!("http://localhost:{port}")
        }
    }
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid header")
}

fn serve(mut request: tiny_http::Request, port: u16, bridge: &Bridge, status: &Mutex<Status>) {
    let origin = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Origin"))
        .map(|h| h.value.as_str().to_owned());
    let path = request
        .url()
        .split('?')
        .next()
        .unwrap_or("")
        .trim_end_matches('/')
        .to_owned();
    let text = |code: u16, body: &str| {
        Response::from_string(body)
            .with_status_code(StatusCode(code))
            .with_header(header("Content-Type", "text/plain; charset=utf-8"))
    };
    let response = if !origin_allowed(origin.as_deref(), port) {
        text(403, "Forbidden origin")
    } else if path != PATH {
        text(404, "Not found: the MCP endpoint is /mcp")
    } else if *request.method() != Method::Post {
        text(405, "Use POST").with_header(header("Allow", "POST"))
    } else {
        let mut body = Vec::new();
        let read = request
            .as_reader()
            .take(MAX_BODY + 1)
            .read_to_end(&mut body);
        if read.is_err() {
            text(400, "Cannot read the request")
        } else if body.len() as u64 > MAX_BODY {
            text(413, "Request too large")
        } else {
            match protocol::handle_body(&body, bridge, status) {
                Some(answer) => Response::from_string(answer.to_string())
                    .with_header(header("Content-Type", "application/json")),
                None => text(202, ""),
            }
        }
    };
    if let Err(err) = request.respond(response) {
        tracing::debug!(%err, "MCP answer not delivered");
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    use super::*;
    use crate::bridge;

    /// A free port on the loopback address.
    pub fn free_port() -> u16 {
        std::net::TcpListener::bind(("127.0.0.1", 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    /// Sends a raw request; returns the status code and body.
    fn send(port: u16, method: &str, headers: &[(&str, &str)], body: &str) -> (u16, String) {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut req = format!(
            "{method} /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        );
        for (k, v) in headers {
            req.push_str(&format!("{k}: {v}\r\n"));
        }
        req.push_str("\r\n");
        req.push_str(body);
        stream.write_all(req.as_bytes()).unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let code = line.split_whitespace().nth(1).unwrap().parse().unwrap();
        let mut rest = String::new();
        let _ = reader.read_to_string(&mut rest);
        let body = rest
            .split_once("\r\n\r\n")
            .map_or("", |(_, b)| b)
            .to_owned();
        (code, body)
    }

    fn running() -> (ServerHandle, bridge::Inbox, Arc<Mutex<Status>>) {
        let (b, inbox) = bridge::bridge(|| {}, Duration::from_millis(100));
        let status = Arc::new(Mutex::new(Status::default()));
        let handle = start(free_port(), b, status.clone()).unwrap();
        (handle, inbox, status)
    }

    #[test]
    fn answers_on_loopback() {
        let (handle, _inbox, status) = running();
        assert_eq!(status.lock().unwrap().state, ServerState::Listening);
        let (code, body) = send(
            handle.port,
            "POST",
            &[("Content-Type", "application/json")],
            r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#,
        );
        assert_eq!(code, 200);
        assert!(body.contains(r#""result":{}"#), "{body}");
        let (code, _) = send(
            handle.port,
            "POST",
            &[],
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        );
        assert_eq!(code, 202);
        let (code, _) = send(handle.port, "GET", &[], "");
        assert_eq!(code, 405);
        handle.stop();
    }

    #[test]
    fn web_page_blocked() {
        let (handle, inbox, _status) = running();
        let (code, _) = send(
            handle.port,
            "POST",
            &[("Origin", "https://example.com")],
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_project"}}"#,
        );
        assert_eq!(code, 403);
        assert!(inbox.next().is_none(), "no call reached the app");
        let local = format!("http://localhost:{}", handle.port);
        let (code, _) = send(
            handle.port,
            "POST",
            &[("Origin", &local)],
            r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#,
        );
        assert_eq!(code, 200);
    }

    #[test]
    fn port_in_use() {
        let taken = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = taken.local_addr().unwrap().port();
        let (b, _inbox) = bridge::bridge(|| {}, Duration::from_millis(100));
        let status = Arc::new(Mutex::new(Status::default()));
        assert!(matches!(
            start(port, b, status.clone()),
            Err(StartError::PortInUse)
        ));
        let s = status.lock().unwrap();
        assert_eq!(s.state, ServerState::PortInUse);
        assert_eq!(s.port, port);
    }

    #[test]
    fn stopped_server_refuses_connections() {
        let (handle, inbox, _status) = running();
        let port = handle.port;
        drop(inbox);
        handle.stop();
        assert!(TcpStream::connect(("127.0.0.1", port)).is_err());
    }
}
