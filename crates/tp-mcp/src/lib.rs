//! Local MCP server of TruckPaint: an LLM client draws on the open project
//! through it. This crate holds the protocol (JSON-RPC over Streamable
//! HTTP on the loopback address), the tools' schemas, and their arguments
//! checked and turned into [`call::Call`]s. It never touches a project: the
//! app takes the calls through [`bridge`] and runs them.

pub mod args;
pub mod bridge;
pub mod call;
pub mod protocol;
pub mod server;
pub mod status;
pub mod tools;

pub use bridge::{Bridge, Inbox, Job, Outcome, Reply, bridge};
pub use call::Call;
pub use server::{DEFAULT_PORT, ServerHandle, StartError, start, url};
pub use status::{ServerState, Status};
