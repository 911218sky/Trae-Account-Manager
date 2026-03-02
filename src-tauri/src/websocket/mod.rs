// WebSocket module
// Manages WebSocket connections with IDE instances

pub mod server;
pub mod types;

pub use server::WebSocketServer;
pub use types::SessionEvent;
