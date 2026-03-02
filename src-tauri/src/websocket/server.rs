// WebSocket Server implementation

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

use super::types::{SessionEvent, WebSocketMessage};

/// WebSocket connection information
#[allow(dead_code)]
pub struct WebSocketConnection {
    pub client_id: String,
    pub sender: mpsc::UnboundedSender<Message>,
    pub last_heartbeat: Instant,
}

/// WebSocket server for IDE instance communication
#[allow(dead_code)]
pub struct WebSocketServer {
    port: u16,
    connections: Arc<Mutex<HashMap<String, WebSocketConnection>>>,
}

impl WebSocketServer {
    /// Start the WebSocket server
    pub async fn start(port: u16) -> Result<Self> {
        let connections = Arc::new(Mutex::new(HashMap::new()));
        let server = Self {
            port,
            connections: connections.clone(),
        };

        // Spawn server task
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr)
            .await
            .context(format!("Failed to bind to {}", addr))?;

        log::info!("WebSocket server listening on {}", addr);

        // Spawn connection handler
        tokio::spawn(async move {
            while let Ok((stream, peer)) = listener.accept().await {
                log::info!("New connection from {}", peer);
                let connections = connections.clone();
                tokio::spawn(handle_connection(stream, peer, connections));
            }
        });

        Ok(server)
    }

    /// Broadcast an event to all connected clients
    pub async fn broadcast(&self, event: SessionEvent) -> Result<usize> {
        let message = serde_json::to_string(&event)?;
        let ws_message = Message::Text(message);

        let connections = self.connections.lock().await;
        let mut sent_count = 0;

        for (client_id, conn) in connections.iter() {
            if let Err(e) = conn.sender.send(ws_message.clone()) {
                log::warn!("Failed to send message to client {}: {}", client_id, e);
            } else {
                sent_count += 1;
            }
        }

        log::info!("Broadcast event to {} clients", sent_count);
        Ok(sent_count)
    }

    /// Run heartbeat loop
    pub async fn heartbeat_loop(&self) {
        let connections = self.connections.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                let mut connections = connections.lock().await;
                let now = Instant::now();
                let timeout = Duration::from_secs(30);
                
                // Send heartbeat and remove stale connections
                let mut to_remove = Vec::new();
                
                for (client_id, conn) in connections.iter() {
                    // Check if connection is stale (no heartbeat response for 30s)
                    if now.duration_since(conn.last_heartbeat) > timeout {
                        println!("[WARN] Client {} timed out, removing connection", client_id);
                        to_remove.push(client_id.clone());
                        continue;
                    }
                    
                    // Send heartbeat
                    let heartbeat = SessionEvent::Heartbeat {
                        timestamp: chrono::Utc::now().timestamp(),
                    };
                    
                    if let Ok(message) = serde_json::to_string(&heartbeat) {
                        if let Err(e) = conn.sender.send(Message::Text(message)) {
                            println!("[WARN] Failed to send heartbeat to {}: {}", client_id, e);
                            to_remove.push(client_id.clone());
                        }
                    }
                }
                
                // Remove stale connections
                for client_id in to_remove {
                    connections.remove(&client_id);
                    println!("[INFO] Removed stale connection: {}", client_id);
                }
            }
        });
    }

    /// Remove a disconnected client
    pub fn remove_connection(&self, client_id: &str) {
        let connections = self.connections.clone();
        let client_id = client_id.to_string();
        
        tokio::spawn(async move {
            let mut connections = connections.lock().await;
            connections.remove(&client_id);
            log::info!("Removed connection: {}", client_id);
        });
    }

    /// Get the number of connected clients
    pub async fn connection_count(&self) -> usize {
        self.connections.lock().await.len()
    }
}

/// Handle a single WebSocket connection
async fn handle_connection(
    stream: TcpStream,
    peer: SocketAddr,
    connections: Arc<Mutex<HashMap<String, WebSocketConnection>>>,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("Failed to accept WebSocket connection from {}: {}", peer, e);
            return;
        }
    };

    log::info!("WebSocket connection established with {}", peer);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Generate unique client ID
    let client_id = Uuid::new_v4().to_string();
    let client_id_clone = client_id.clone();

    // Store connection
    {
        let mut conns = connections.lock().await;
        conns.insert(
            client_id.clone(),
            WebSocketConnection {
                client_id: client_id.clone(),
                sender: tx,
                last_heartbeat: Instant::now(),
            },
        );
    }

    log::info!("Client registered with ID: {}", client_id);

    // Spawn task to send messages to client
    let send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = ws_sender.send(message).await {
                log::error!("Failed to send message to {}: {}", peer, e);
                break;
            }
        }
    });

    // Handle incoming messages from client
    let connections_clone = connections.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    // Parse message
                    if let Ok(msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                        match msg {
                            WebSocketMessage::HeartbeatAck { client_id: ack_id, .. } => {
                                // Update last heartbeat time
                                let mut conns = connections_clone.lock().await;
                                if let Some(conn) = conns.get_mut(&ack_id) {
                                    conn.last_heartbeat = Instant::now();
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    log::info!("Client {} closed connection", client_id_clone);
                    break;
                }
                Ok(Message::Ping(_)) => {
                    // Pong is automatically sent by tokio-tungstenite
                }
                Ok(Message::Pong(_)) => {}
                Err(e) => {
                    log::error!("WebSocket error for {}: {}", client_id_clone, e);
                    break;
                }
                _ => {}
            }
        }

        // Remove connection when done
        let mut conns = connections_clone.lock().await;
        conns.remove(&client_id_clone);
        log::info!("Connection removed: {}", client_id_clone);
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_websocket_server_starts() {
        // Test that the server can start on a port
        let result = WebSocketServer::start(9528).await;
        assert!(result.is_ok());
        
        let server = result.unwrap();
        assert_eq!(server.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_broadcast_with_no_connections() {
        // Test broadcasting when no clients are connected
        let server = WebSocketServer::start(9529).await.unwrap();
        
        let event = SessionEvent::SessionChanged {
            account_id: "test_account".to_string(),
            user_id: "test_user".to_string(),
            email: "test@example.com".to_string(),
            token: "test_token".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        let result = server.broadcast(event).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // No clients connected
    }

    #[tokio::test]
    async fn test_heartbeat_loop_starts() {
        // Test that heartbeat loop can be started
        let server = WebSocketServer::start(9530).await.unwrap();
        server.heartbeat_loop().await;
        
        // Give it a moment to start
        sleep(Duration::from_millis(100)).await;
        
        // If we get here without panic, the heartbeat loop started successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_remove_connection() {
        // Test that we can remove a connection
        let server = WebSocketServer::start(9531).await.unwrap();
        
        // Add a fake connection
        {
            let (tx, _rx) = mpsc::unbounded_channel();
            let mut conns = server.connections.lock().await;
            conns.insert(
                "test_client".to_string(),
                WebSocketConnection {
                    client_id: "test_client".to_string(),
                    sender: tx,
                    last_heartbeat: Instant::now(),
                },
            );
        }
        
        assert_eq!(server.connection_count().await, 1);
        
        // Remove the connection
        server.remove_connection("test_client");
        
        // Give it a moment to process
        sleep(Duration::from_millis(100)).await;
        
        assert_eq!(server.connection_count().await, 0);
    }
}
