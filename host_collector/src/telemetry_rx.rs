use serde::Deserialize;
use postcard::from_bytes_cobs;
use std::io::Read;
use serialport::SerialPort;

#[derive(Deserialize, Debug)]
pub struct Telemetry {
    pub temp: f32,
    pub uptime_ms: u32,
    pub interval_ms: u32,
}

pub fn run_telemetry_loop(mut port: Box<dyn SerialPort>) {
    let mut packet_buffer = Vec::new();

    loop {
        let mut byte = [0u8; 1];
        
        // Guard: Skip if read fails
        if port.read_exact(&mut byte).is_err() {
            continue;
        }

        let b = byte[0];

        // Case: Data byte (Not 0) -> Save and continue
        if b != 0 {
            packet_buffer.push(b);
            continue;
        }

        // Case: Frame End (is 0)
        if packet_buffer.is_empty() {
            continue;
        }

        // Process the complete packet
        match from_bytes_cobs::<Telemetry>(&mut packet_buffer) {
            Ok(data) => {
                println!(
                    "[Uptime: {:5}ms] Internal Temp: {:.2}°C (Interval: {}ms)",
                    data.uptime_ms, data.temp, data.interval_ms
                );
            }
            Err(_) => {
                // Ignore noise
            }
        }
        
        packet_buffer.clear();
    }
}
