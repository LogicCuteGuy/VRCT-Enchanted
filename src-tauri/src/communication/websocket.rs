// WebSocket server implementation
use crate::utils::error::{Result, VrctError};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

/// WebSocket message structure for broadcasting events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
}

/// WebSocket server state shared across connections
#[derive(Clone)]
struct ServerState {
    /// Broadcast channel for sending messages to all connected clients
    tx: broadcast::Sender<String>,
}

/// WebSocket server for external integrations
pub struct WebSocketServer {
    /// Server address (host:port)
    addr: Arc<RwLock<Option<SocketAddr>>>,
    /// Broadcast sender for messages
    tx: broadcast::Sender<String>,
    /// Server task handle
    server_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    /// Shutdown signal sender
    shutdown_tx: Arc<RwLock<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new() -> Self {
        // Create broadcast channel with capacity for 100 messages
        let (tx, _) = broadcast::channel(100);
        
        Self {
            addr: Arc::new(RwLock::new(None)),
            tx,
            server_handle: Arc::new(RwLock::new(None)),
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Start the WebSocket server on the specified address
    pub async fn start(&self, addr: SocketAddr) -> Result<()> {
        // Stop existing server if running
        self.stop().await;
        
        info!("Starting WebSocket server on {}", addr);
        
        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        
        // Store the shutdown sender
        *self.shutdown_tx.write().await = Some(shutdown_tx);
        
        // Create server state
        let state = ServerState {
            tx: self.tx.clone(),
        };
        
        // Build the router
        let app = Router::new()
            .route("/", get(websocket_handler))
            .with_state(state);
        
        // Spawn server task
        let server_handle = tokio::spawn(async move {
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => {
                    info!("WebSocket server listening on {}", addr);
                    listener
                }
                Err(e) => {
                    error!("Failed to bind WebSocket server to {}: {}", addr, e);
                    return;
                }
            };
            
            // Run server with graceful shutdown
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    shutdown_rx.await.ok();
                    info!("WebSocket server shutdown signal received");
                })
                .await
                .unwrap_or_else(|e| {
                    error!("WebSocket server error: {}", e);
                });
            
            info!("WebSocket server stopped");
        });
        
        // Store server handle and address
        *self.server_handle.write().await = Some(server_handle);
        *self.addr.write().await = Some(addr);
        
        Ok(())
    }
    
    /// Stop the WebSocket server
    pub async fn stop(&self) {
        let mut shutdown_tx = self.shutdown_tx.write().await;
        if let Some(tx) = shutdown_tx.take() {
            info!("Stopping WebSocket server");
            let _ = tx.send(());
        }
        
        let mut handle = self.server_handle.write().await;
        if let Some(h) = handle.take() {
            // Wait for server to shut down
            let _ = h.await;
        }
        
        *self.addr.write().await = None;
    }
    
    /// Broadcast a message to all connected clients
    pub async fn broadcast(&self, message: &WebSocketMessage) -> Result<()> {
        let json = serde_json::to_string(message)
            .map_err(|e| VrctError::WebSocket(format!("Failed to serialize message: {}", e)))?;
        
        // Send to broadcast channel
        // Ignore error if no receivers (no clients connected)
        let _ = self.tx.send(json);
        
        debug!("Broadcast message: {}", message.message_type);
        Ok(())
    }
    
    /// Check if the server is currently running
    pub async fn is_alive(&self) -> bool {
        let handle = self.server_handle.read().await;
        if let Some(h) = handle.as_ref() {
            !h.is_finished()
        } else {
            false
        }
    }
    
    /// Get the current server address
    pub async fn get_address(&self) -> Option<SocketAddr> {
        *self.addr.read().await
    }
    
    /// Restart the server with a new address
    pub async fn restart(&self, new_addr: SocketAddr) -> Result<()> {
        info!("Restarting WebSocket server on {}", new_addr);
        self.stop().await;
        self.start(new_addr).await
    }
}

impl Default for WebSocketServer {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket connection handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: ServerState) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();
    
    info!("New WebSocket client connected");
    
    // Spawn task to forward broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
    
    // Handle incoming messages from client (currently just echo/ignore)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    debug!("Received text message from client: {}", text);
                }
                Message::Binary(data) => {
                    debug!("Received binary message from client: {} bytes", data.len());
                }
                Message::Ping(_) => {
                    debug!("Received ping from client");
                }
                Message::Pong(_) => {
                    debug!("Received pong from client");
                }
                Message::Close(_) => {
                    info!("Client sent close message");
                    break;
                }
            }
        }
    });
    
    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }
    
    info!("WebSocket client disconnected");
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_websocket_server_creation() {
        let server = WebSocketServer::new();
        assert!(!server.is_alive().await);
        assert!(server.get_address().await.is_none());
    }
    
    #[tokio::test]
    async fn test_websocket_server_start_stop() {
        let server = WebSocketServer::new();
        let addr: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        
        // Start server
        server.start(addr).await.expect("Failed to start server");
        
        // Give server time to start
        sleep(Duration::from_millis(100)).await;
        
        assert!(server.is_alive().await);
        assert_eq!(server.get_address().await, Some(addr));
        
        // Stop server
        server.stop().await;
        
        // Give server time to stop
        sleep(Duration::from_millis(100)).await;
        
        assert!(!server.is_alive().await);
        assert!(server.get_address().await.is_none());
    }
    
    #[tokio::test]
    async fn test_websocket_server_restart() {
        let server = WebSocketServer::new();
        let addr1: SocketAddr = "127.0.0.1:9002".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:9003".parse().unwrap();
        
        // Start on first address
        server.start(addr1).await.expect("Failed to start server");
        sleep(Duration::from_millis(100)).await;
        assert_eq!(server.get_address().await, Some(addr1));
        
        // Restart on second address
        server.restart(addr2).await.expect("Failed to restart server");
        sleep(Duration::from_millis(100)).await;
        assert_eq!(server.get_address().await, Some(addr2));
        
        // Cleanup
        server.stop().await;
    }
    
    #[tokio::test]
    async fn test_websocket_broadcast() {
        let server = WebSocketServer::new();
        
        let message = WebSocketMessage {
            message_type: "test".to_string(),
            data: serde_json::json!({"key": "value"}),
        };
        
        // Should not error even if no clients connected
        server.broadcast(&message).await.expect("Broadcast failed");
    }
}

    
    #[tokio::test]
    async fn test_websocket_client_connection_and_broadcast() {
        use tokio_tungstenite::connect_async;
        use tokio::time::{sleep, Duration};
        
        let server = WebSocketServer::new();
        let addr: SocketAddr = "127.0.0.1:9004".parse().unwrap();
        
        // Start server
        server.start(addr).await.expect("Failed to start server");
        sleep(Duration::from_millis(200)).await;
        
        // Connect a client
        let url = format!("ws://{}", addr);
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
        let (mut _write, mut read) = ws_stream.split();
        
        // Broadcast a message
        let message = WebSocketMessage {
            message_type: "transcription".to_string(),
            data: serde_json::json!({
                "text": "Hello, world!",
                "language": "en"
            }),
        };
        
        server.broadcast(&message).await.expect("Broadcast failed");
        
        // Read the message from client
        if let Some(Ok(msg)) = read.next().await {
            if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                let received: WebSocketMessage = serde_json::from_str(&text)
                    .expect("Failed to parse message");
                assert_eq!(received.message_type, "transcription");
                assert_eq!(received.data["text"], "Hello, world!");
                assert_eq!(received.data["language"], "en");
            } else {
                panic!("Expected text message");
            }
        } else {
            panic!("No message received");
        }
        
        // Cleanup
        server.stop().await;
    }
