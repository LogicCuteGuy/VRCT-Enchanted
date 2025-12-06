// OSC protocol implementation for VRChat communication
// This module provides OSC message sending and receiving capabilities

use crate::utils::error::{Result, VrctError};
use rosc::{OscMessage, OscPacket, OscType};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Callback type for OSC parameter changes
pub type OscCallback = Arc<dyn Fn(String, Vec<OscType>) + Send + Sync>;

/// OSC handler for sending and receiving OSC messages
/// Manages communication with VRChat via OSC protocol
pub struct OscHandler {
    /// Target IP address for sending OSC messages
    ip_address: String,
    /// Target port for sending OSC messages
    port: u16,
    /// UDP socket for sending messages
    sender: Arc<RwLock<Option<UdpSocket>>>,
    /// UDP socket for receiving messages (optional)
    receiver: Arc<RwLock<Option<UdpSocket>>>,
    /// Port for receiving OSC messages
    receiver_port: Option<u16>,
    /// Handle for the receiver task
    receiver_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    /// Flag to signal receiver shutdown
    shutdown_flag: Arc<RwLock<bool>>,
}

impl OscHandler {
    /// VRChat OSC parameter paths
    pub const PARAM_MUTE_SELF: &'static str = "/avatar/parameters/MuteSelf";
    pub const PARAM_CHATBOX_TYPING: &'static str = "/chatbox/typing";
    pub const PARAM_CHATBOX_INPUT: &'static str = "/chatbox/input";

    /// Create a new OSC handler with the specified IP address and port
    ///
    /// # Arguments
    /// * `ip_address` - Target IP address (e.g., "127.0.0.1")
    /// * `port` - Target UDP port (e.g., 9000)
    ///
    /// # Returns
    /// A Result containing the OscHandler or an error
    pub fn new(ip_address: &str, port: u16) -> Result<Self> {
        info!("Creating OSC handler for {}:{}", ip_address, port);
        
        let handler = Self {
            ip_address: ip_address.to_string(),
            port,
            sender: Arc::new(RwLock::new(None)),
            receiver: Arc::new(RwLock::new(None)),
            receiver_port: None,
            receiver_task: Arc::new(RwLock::new(None)),
            shutdown_flag: Arc::new(RwLock::new(false)),
        };

        Ok(handler)
    }

    /// Initialize the OSC sender socket
    async fn ensure_sender(&self) -> Result<()> {
        let mut sender = self.sender.write().await;
        
        if sender.is_none() {
            let socket = UdpSocket::bind("0.0.0.0:0")
                .map_err(|e| VrctError::Osc(format!("Failed to bind sender socket: {}", e)))?;
            
            socket.set_nonblocking(false)
                .map_err(|e| VrctError::Osc(format!("Failed to set socket to blocking: {}", e)))?;
            
            debug!("OSC sender socket initialized");
            *sender = Some(socket);
        }
        
        Ok(())
    }

    /// Get the target socket address for sending messages
    fn get_target_addr(&self) -> Result<SocketAddr> {
        let addr_str = format!("{}:{}", self.ip_address, self.port);
        addr_str.parse()
            .map_err(|e| VrctError::Osc(format!("Invalid OSC address {}: {}", addr_str, e)))
    }

    /// Send an OSC packet to the configured address
    async fn send_packet(&self, packet: OscPacket) -> Result<()> {
        self.ensure_sender().await?;
        
        let target_addr = self.get_target_addr()?;
        let sender = self.sender.read().await;
        
        if let Some(socket) = sender.as_ref() {
            let buf = rosc::encoder::encode(&packet)
                .map_err(|e| VrctError::Osc(format!("Failed to encode OSC packet: {}", e)))?;
            
            socket.send_to(&buf, target_addr)
                .map_err(|e| VrctError::Osc(format!("Failed to send OSC packet: {}", e)))?;
            
            debug!("Sent OSC packet to {}", target_addr);
            Ok(())
        } else {
            Err(VrctError::Osc("Sender socket not initialized".to_string()))
        }
    }

    /// Update the target IP address
    ///
    /// # Arguments
    /// * `ip_address` - New target IP address
    pub async fn set_ip_address(&mut self, ip_address: &str) -> Result<()> {
        info!("Changing OSC IP address to {}", ip_address);
        self.ip_address = ip_address.to_string();
        
        // Reset sender to force reconnection with new address
        let mut sender = self.sender.write().await;
        *sender = None;
        
        Ok(())
    }

    /// Update the target port
    ///
    /// # Arguments
    /// * `port` - New target UDP port
    pub async fn set_port(&mut self, port: u16) -> Result<()> {
        info!("Changing OSC port to {}", port);
        self.port = port;
        
        // Reset sender to force reconnection with new port
        let mut sender = self.sender.write().await;
        *sender = None;
        
        Ok(())
    }

    /// Check if OSC Query is enabled (only for localhost addresses)
    pub fn is_osc_query_enabled(&self) -> bool {
        self.ip_address == "127.0.0.1" || self.ip_address == "localhost"
    }

    /// Send a message to VRChat chatbox
    ///
    /// # Arguments
    /// * `message` - The message text to send
    /// * `notification` - Whether to trigger a notification sound in VRChat
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn send_message(&self, message: &str, notification: bool) -> Result<()> {
        if message.is_empty() {
            debug!("Skipping empty message");
            return Ok(());
        }

        let msg = OscMessage {
            addr: Self::PARAM_CHATBOX_INPUT.to_string(),
            args: vec![
                OscType::String(message.to_string()),
                OscType::Bool(true),  // immediate send flag
                OscType::Bool(notification),
            ],
        };

        let packet = OscPacket::Message(msg);
        self.send_packet(packet).await?;
        
        debug!("Sent chatbox message: {} (notification: {})", message, notification);
        Ok(())
    }

    /// Send typing indicator to VRChat
    ///
    /// # Arguments
    /// * `is_typing` - True to show typing indicator, false to hide it
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn send_typing(&self, is_typing: bool) -> Result<()> {
        let msg = OscMessage {
            addr: Self::PARAM_CHATBOX_TYPING.to_string(),
            args: vec![OscType::Bool(is_typing)],
        };

        let packet = OscPacket::Message(msg);
        self.send_packet(packet).await?;
        
        debug!("Sent typing indicator: {}", is_typing);
        Ok(())
    }

    /// Start receiving OSC parameters from VRChat
    ///
    /// # Arguments
    /// * `callback` - Function to call when OSC messages are received
    /// * `bind_port` - Optional port to bind to (defaults to 9001)
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn start_receiver<F>(&mut self, callback: F, bind_port: Option<u16>) -> Result<()>
    where
        F: Fn(String, Vec<OscType>) + Send + Sync + 'static,
    {
        // Stop any existing receiver
        self.stop_receiver().await?;

        let port = bind_port.unwrap_or(9001);
        let bind_addr = format!("0.0.0.0:{}", port);
        
        info!("Starting OSC receiver on {}", bind_addr);

        let socket = UdpSocket::bind(&bind_addr)
            .map_err(|e| VrctError::Osc(format!("Failed to bind receiver socket: {}", e)))?;
        
        socket.set_nonblocking(true)
            .map_err(|e| VrctError::Osc(format!("Failed to set socket to non-blocking: {}", e)))?;

        self.receiver_port = Some(port);
        *self.receiver.write().await = Some(socket);
        *self.shutdown_flag.write().await = false;

        // Spawn receiver task
        let receiver = self.receiver.clone();
        let shutdown_flag = self.shutdown_flag.clone();
        let callback: OscCallback = Arc::new(callback);

        let task = tokio::spawn(async move {
            let mut buf = [0u8; 65536];
            
            loop {
                // Check shutdown flag
                if *shutdown_flag.read().await {
                    debug!("OSC receiver shutting down");
                    break;
                }

                let socket_guard = receiver.read().await;
                if let Some(socket) = socket_guard.as_ref() {
                    match socket.recv_from(&mut buf) {
                        Ok((size, _addr)) => {
                            match rosc::decoder::decode_udp(&buf[..size]) {
                                Ok((_, packet)) => {
                                    Self::handle_packet(&packet, &callback);
                                }
                                Err(e) => {
                                    warn!("Failed to decode OSC packet: {}", e);
                                }
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // No data available, sleep briefly
                            drop(socket_guard);
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        }
                        Err(e) => {
                            error!("Error receiving OSC data: {}", e);
                            drop(socket_guard);
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                } else {
                    break;
                }
            }
            
            info!("OSC receiver task ended");
        });

        *self.receiver_task.write().await = Some(task);
        
        Ok(())
    }

    /// Handle an incoming OSC packet
    fn handle_packet(packet: &OscPacket, callback: &OscCallback) {
        match packet {
            OscPacket::Message(msg) => {
                debug!("Received OSC message: {} with {} args", msg.addr, msg.args.len());
                callback(msg.addr.clone(), msg.args.clone());
            }
            OscPacket::Bundle(bundle) => {
                debug!("Received OSC bundle with {} messages", bundle.content.len());
                for packet in &bundle.content {
                    Self::handle_packet(packet, callback);
                }
            }
        }
    }

    /// Stop the OSC receiver
    pub async fn stop_receiver(&mut self) -> Result<()> {
        info!("Stopping OSC receiver");
        
        // Signal shutdown
        *self.shutdown_flag.write().await = true;

        // Wait for task to complete
        let mut task_guard = self.receiver_task.write().await;
        if let Some(task) = task_guard.take() {
            // Give it a moment to shut down gracefully
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            task.abort();
        }

        // Close socket
        *self.receiver.write().await = None;
        self.receiver_port = None;

        Ok(())
    }

    /// Get the current receiver port
    pub fn get_receiver_port(&self) -> Option<u16> {
        self.receiver_port
    }

    /// Query an OSC parameter value using OSC Query protocol
    ///
    /// This is a placeholder for OSC Query support. Full implementation would require:
    /// - mDNS/Zeroconf service discovery
    /// - HTTP client for OSC Query JSON API
    /// - Service browser to find VRChat-Client service
    ///
    /// # Arguments
    /// * `parameter` - The OSC parameter path to query (e.g., "/avatar/parameters/MuteSelf")
    ///
    /// # Returns
    /// The parameter value if available, None otherwise
    pub async fn query_parameter(&self, parameter: &str) -> Option<OscType> {
        if !self.is_osc_query_enabled() {
            debug!("OSC Query not enabled for non-localhost addresses");
            return None;
        }

        // TODO: Implement full OSC Query support
        // This would involve:
        // 1. Using mDNS to discover VRChat-Client service
        // 2. Making HTTP requests to the OSC Query JSON API
        // 3. Parsing the response to get parameter values
        
        warn!("OSC Query parameter discovery not yet implemented for: {}", parameter);
        None
    }

    /// Get the MuteSelf parameter value from VRChat
    ///
    /// This is a convenience method for querying the mic mute state
    ///
    /// # Returns
    /// True if muted, False if not muted, None if unavailable
    pub async fn get_mute_self(&self) -> Option<bool> {
        match self.query_parameter(Self::PARAM_MUTE_SELF).await {
            Some(OscType::Bool(value)) => Some(value),
            Some(OscType::Int(value)) => Some(value != 0),
            Some(OscType::Float(value)) => Some(value != 0.0),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_osc_handler_creation() {
        let handler = OscHandler::new("127.0.0.1", 9000);
        assert!(handler.is_ok());
        
        let handler = handler.unwrap();
        assert_eq!(handler.ip_address, "127.0.0.1");
        assert_eq!(handler.port, 9000);
    }

    #[tokio::test]
    async fn test_osc_query_enabled() {
        let handler = OscHandler::new("127.0.0.1", 9000).unwrap();
        assert!(handler.is_osc_query_enabled());
        
        let handler = OscHandler::new("localhost", 9000).unwrap();
        assert!(handler.is_osc_query_enabled());
        
        let handler = OscHandler::new("192.168.1.1", 9000).unwrap();
        assert!(!handler.is_osc_query_enabled());
    }

    #[tokio::test]
    async fn test_set_ip_address() {
        let mut handler = OscHandler::new("127.0.0.1", 9000).unwrap();
        assert_eq!(handler.ip_address, "127.0.0.1");
        
        handler.set_ip_address("192.168.1.1").await.unwrap();
        assert_eq!(handler.ip_address, "192.168.1.1");
    }

    #[tokio::test]
    async fn test_set_port() {
        let mut handler = OscHandler::new("127.0.0.1", 9000).unwrap();
        assert_eq!(handler.port, 9000);
        
        handler.set_port(9001).await.unwrap();
        assert_eq!(handler.port, 9001);
    }

    #[tokio::test]
    async fn test_send_message_empty() {
        let handler = OscHandler::new("127.0.0.1", 9000).unwrap();
        // Empty messages should be skipped without error
        let result = handler.send_message("", true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_typing() {
        let handler = OscHandler::new("127.0.0.1", 9000).unwrap();
        // Should not error even if no receiver is listening
        let result = handler.send_typing(true).await;
        assert!(result.is_ok());
        
        let result = handler.send_typing(false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_receiver_start_stop() {
        let mut handler = OscHandler::new("127.0.0.1", 9000).unwrap();
        
        let callback = |addr: String, args: Vec<OscType>| {
            println!("Received: {} with {} args", addr, args.len());
        };
        
        // Start receiver
        let result = handler.start_receiver(callback, Some(19001)).await;
        assert!(result.is_ok());
        assert_eq!(handler.get_receiver_port(), Some(19001));
        
        // Stop receiver
        let result = handler.stop_receiver().await;
        assert!(result.is_ok());
        assert_eq!(handler.get_receiver_port(), None);
    }

    #[tokio::test]
    async fn test_receiver_message_handling() {
        use std::sync::atomic::{AtomicBool, Ordering};
        
        let mut handler = OscHandler::new("127.0.0.1", 9000).unwrap();
        let received = Arc::new(AtomicBool::new(false));
        let received_clone = received.clone();
        
        let callback = move |addr: String, _args: Vec<OscType>| {
            if addr == OscHandler::PARAM_MUTE_SELF {
                received_clone.store(true, Ordering::SeqCst);
            }
        };
        
        // Start receiver on a specific port
        handler.start_receiver(callback, Some(19002)).await.unwrap();
        
        // Create a sender to send to our receiver
        let sender_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let msg = OscMessage {
            addr: OscHandler::PARAM_MUTE_SELF.to_string(),
            args: vec![OscType::Bool(true)],
        };
        let packet = OscPacket::Message(msg);
        let buf = rosc::encoder::encode(&packet).unwrap();
        sender_socket.send_to(&buf, "127.0.0.1:19002").unwrap();
        
        // Wait a bit for the message to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Check if message was received
        assert!(received.load(Ordering::SeqCst));
        
        handler.stop_receiver().await.unwrap();
    }

    #[tokio::test]
    async fn test_osc_query_placeholder() {
        let handler = OscHandler::new("127.0.0.1", 9000).unwrap();
        
        // OSC Query is not fully implemented yet, should return None
        let result = handler.query_parameter(OscHandler::PARAM_MUTE_SELF).await;
        assert!(result.is_none());
        
        let result = handler.get_mute_self().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_osc_query_disabled_for_remote() {
        let handler = OscHandler::new("192.168.1.1", 9000).unwrap();
        
        // OSC Query should be disabled for non-localhost
        assert!(!handler.is_osc_query_enabled());
        let result = handler.query_parameter(OscHandler::PARAM_MUTE_SELF).await;
        assert!(result.is_none());
    }
}
