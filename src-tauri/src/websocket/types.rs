// WebSocket data types

use serde::{Deserialize, Serialize};

/// Session events sent to IDE instances
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SessionEvent {
    #[serde(rename = "session_changed")]
    SessionChanged {
        account_id: String,
        user_id: String,
        email: String,
        token: String,
        timestamp: i64,
    },
    #[serde(rename = "account_updated")]
    AccountUpdated {
        account_id: String,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        timestamp: i64,
    },
}

/// WebSocket messages
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "session_changed")]
    SessionChanged {
        account_id: String,
        user_id: String,
        email: String,
        token: String,
        timestamp: i64,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        timestamp: i64,
    },
    #[serde(rename = "heartbeat_ack")]
    HeartbeatAck {
        client_id: String,
        timestamp: i64,
    },
    #[serde(rename = "error")]
    Error {
        code: String,
        message: String,
    },
}
