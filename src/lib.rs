#![no_std]

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Telemetry {
    pub temp: f32,
    pub uptime_ms: u32,
    pub interval_ms: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Command {
    SetInterval { millis: u32 },
    ToggleInterval,
}
