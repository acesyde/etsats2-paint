//! The local MCP server in the app: started and stopped from Preferences,
//! and its calls run on the project between frames, never during a
//! gesture or under a dialog (see the `tp-mcp` crate for the protocol).

mod read;
mod write;

pub use write::EDITING_SYMBOL;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use tp_mcp::{Call, Inbox, Reply, ServerHandle, ServerState, Status};

use crate::state::{AppState, Modal};

/// The server and its calls waiting to run.
#[derive(Default)]
pub struct McpHost {
    status: Arc<Mutex<Status>>,
    server: Option<ServerHandle>,
    inbox: Option<Inbox>,
    /// The port a start last failed on, so it isn't retried every frame
    /// (cleared when the setting is turned off or the port changes).
    failed_port: Option<u16>,
    /// How long a call waits for the app (shorter in tests); applies to
    /// the next start.
    pub timeout: Option<std::time::Duration>,
}

/// What the status bar and Preferences show.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpView {
    pub state: ServerState,
    pub port: u16,
    /// A client called in the last minute.
    pub connected: bool,
    /// Changes applied in the last minute.
    pub recent_writes: usize,
}

impl McpHost {
    /// Starts or stops the server to follow the setting.
    pub fn sync(&mut self, enabled: bool, port: u16, ctx: &egui::Context) {
        let running_port = self.server.as_ref().map(|s| s.port);
        if !enabled || running_port.is_some_and(|p| p != port) {
            self.stop();
            if !enabled {
                self.failed_port = None;
                return;
            }
        }
        if self.server.is_some() || self.failed_port == Some(port) {
            return;
        }
        let wake = ctx.clone();
        let (bridge, inbox) = tp_mcp::bridge(
            move || wake.request_repaint(),
            self.timeout.unwrap_or(tp_mcp::bridge::DEFAULT_TIMEOUT),
        );
        match tp_mcp::start(port, bridge, self.status.clone()) {
            Ok(server) => {
                self.server = Some(server);
                self.inbox = Some(inbox);
                self.failed_port = None;
            }
            Err(_) => self.failed_port = Some(port),
        }
    }

    /// Stops the server: calls waiting fail at once.
    pub fn stop(&mut self) {
        // The inbox first, so a call waiting for an answer ends now.
        self.inbox = None;
        if let Some(server) = self.server.take() {
            server.stop();
        }
        if let Ok(mut s) = self.status.lock() {
            s.reset();
        }
    }

    /// The server's state for the interface.
    pub fn view(&self) -> McpView {
        let now = Instant::now();
        let s = self.status.lock().map(|s| s.clone()).unwrap_or_default();
        McpView {
            connected: s.state == ServerState::Listening && s.connected(now),
            recent_writes: s.recent_writes(now),
            state: s.state,
            port: s.port,
        }
    }

    /// When the status changes by itself next (counts expire).
    pub fn next_change(&self) -> Option<std::time::Duration> {
        self.status
            .lock()
            .ok()
            .and_then(|s| s.next_change(Instant::now()))
    }

    /// Whether the server listens.
    pub fn is_running(&self) -> bool {
        self.server.is_some()
    }
}

impl AppState {
    /// Whether calls must wait: a gesture, a text being edited, a pen path,
    /// a live panel edit, or a dialog that changes the document is in
    /// progress. Preferences, About, the shortcuts and messages don't stop
    /// calls.
    pub fn mcp_busy(&self) -> bool {
        let dialog = match &self.modal {
            None
            | Some(
                Modal::Preferences
                | Modal::About
                | Modal::KeyboardShortcuts
                | Modal::Message { .. },
            ) => false,
            Some(_) => true,
        };
        dialog
            || self.workspace().is_some_and(|ws| {
                ws.gesture.is_active()
                    || ws.is_editing_text()
                    || ws.is_drawing_pen()
                    || ws.has_pending()
            })
    }

    /// Runs the calls waiting, unless the user is busy (they wait).
    pub fn run_mcp_calls(&mut self, now: f64) {
        if self.mcp_busy() {
            return;
        }
        let Some(inbox) = &self.mcp.inbox else {
            return;
        };
        let jobs: Vec<_> = std::iter::from_fn(|| inbox.next()).collect();
        for job in jobs {
            if !job.claim() {
                continue;
            }
            let reply = self.run_mcp_call(&job.call, now);
            job.answer(reply);
        }
    }

    /// Runs one call on the open project.
    pub fn run_mcp_call(&mut self, call: &Call, now: f64) -> Reply {
        let Some(ws) = self.workspace_mut() else {
            return Err(NO_PROJECT.to_owned());
        };
        match call {
            Call::GetProject => read::project(ws),
            Call::ListObjects { texture } => read::list_objects(ws, texture),
            Call::GetObject { texture, id } => read::object(ws, texture, *id),
            Call::GetTemplate { texture, max_size } => read::template(ws, texture, *max_size),
            Call::RenderTexture {
                texture,
                max_size,
                with_template,
            } => read::render(ws, texture, *max_size, *with_template),
            Call::AddSwatch { name, color } => write::add_swatch(ws, name, *color, now),
            Call::Write { texture, op } => write::run(ws, texture, op, now),
        }
    }
}

/// What every call fails with on the Start screen.
pub const NO_PROJECT: &str =
    "No project is open in TruckPaint. Ask the user to open or create a project first.";
