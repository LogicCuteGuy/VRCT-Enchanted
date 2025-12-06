// Audio subsystem module
// This module handles audio device management, recording, and energy detection

pub mod device_manager;
pub mod recorder;
pub mod energy;

#[cfg(test)]
mod integration_tests;

pub use device_manager::*;
pub use recorder::*;
pub use energy::*;
