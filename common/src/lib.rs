//! # Common Protocol Definitions
//! 
//! This crate contains the shared data structures used by both the firmware (ESP32)
//! and the station (Mac). By sharing these definitions, we ensure that both sides
//! always agree on the binary format of the data being sent.

#![no_std]
use serde::{Deserialize, Serialize};

/// Data structure sent from the Robot to the Station.
/// Contains internal chip state and timing information.
#[derive(Serialize, Deserialize, Debug)]
pub struct Telemetry {
    /// Internal silicon temperature in Celsius.
    pub temp: f32,
    /// Time since the robot started in milliseconds.
    pub uptime_ms: u32,
    /// The current logging frequency requested by the station.
    pub interval_ms: u32,
}

/// Commands sent from the Station to the Robot.
/// Used to change robot behavior live without reflashing.
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    /// Change the telemetry streaming frequency.
    SetInterval { millis: u32 },
    /// Future command placeholder.
    ToggleInterval,
}
