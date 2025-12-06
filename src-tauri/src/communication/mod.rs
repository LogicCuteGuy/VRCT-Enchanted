// Communication subsystem module
// This module handles OSC and WebSocket communication

pub mod osc;
pub mod websocket;

pub use osc::*;
pub use websocket::{WebSocketMessage, WebSocketServer};
