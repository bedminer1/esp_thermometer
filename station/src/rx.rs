//! # Telemetry Decoder Module
//! 
//! Receives raw bytes from the Serial port, detects packet frames using COBS,
//! and decodes them into human-readable telemetry logs.

use common::Telemetry;
use postcard::from_bytes_cobs;
use std::io::Read;
use serialport::SerialPort;

/// Infinite loop that reads from the serial port and prints decoded telemetry.
/// 
/// Uses an early-continue pattern to efficiently discard noise and wait for
/// the '0' delimiter.
pub fn run_telemetry_loop(mut port: Box<dyn SerialPort>) {
    let mut packet_buffer = Vec::new();

    loop {
        let mut byte = [0u8; 1];
        // Blocking read with a timeout (configured in main.rs)
        if port.read_exact(&mut byte).is_err() {
            continue;
        }

        let b = byte[0];
        
        // Case: Data byte
        if b != 0 {
            packet_buffer.push(b);
            continue;
        }

        // Case: Frame end (b == 0)
        if packet_buffer.is_empty() {
            continue;
        }

        // Decode the completed frame
        match from_bytes_cobs::<Telemetry>(&mut packet_buffer) {
            Ok(data) => {
                println!(
                    "[Uptime: {:5}ms] Internal Temp: {:.2}°C (Interval: {}ms)",
                    data.uptime_ms, data.temp, data.interval_ms
                );
            }
            Err(_) => {
                // Ignore malformed packets (common during initial sync)
            }
        }
        
        packet_buffer.clear();
    }
}
