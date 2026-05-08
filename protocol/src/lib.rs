#![no_std]
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct DownlinkPacket {
    pub temp: f32,
    pub uptime_ms: u32,
    pub interval_ms: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum UplinkCommand {
    SetInterval { millis: u32 },
    ToggleInterval,
}
