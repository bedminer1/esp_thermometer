//! # Firmware Library Hub
//! 
//! This module acts as the internal bridge for the ESP32-C6 firmware.
//! It handles module visibility and forwards shared types from the `common` crate.

#![no_std]

pub mod rx;
pub mod tx;

// Re-export types from the common crate so main.rs can use them directly
pub use common::{Command, Telemetry};
