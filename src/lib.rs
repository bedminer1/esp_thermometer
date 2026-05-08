#![no_std]

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Telemetry {
    pub temp: f32,
    pub uptime_ms: u32,
}
