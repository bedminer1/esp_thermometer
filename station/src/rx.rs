use common::Telemetry;
use postcard::from_bytes_cobs;
use std::io::Read;
use serialport::SerialPort;

pub fn run_telemetry_loop(mut port: Box<dyn SerialPort>) {
    let mut packet_buffer = Vec::new();

    loop {
        let mut byte = [0u8; 1];
        if port.read_exact(&mut byte).is_err() {
            continue;
        }

        let b = byte[0];
        if b != 0 {
            packet_buffer.push(b);
            continue;
        }

        if packet_buffer.is_empty() {
            continue;
        }

        match from_bytes_cobs::<Telemetry>(&mut packet_buffer) {
            Ok(data) => {
                println!(
                    "[Uptime: {:5}ms] Internal Temp: {:.2}°C (Interval: {}ms)",
                    data.uptime_ms, data.temp, data.interval_ms
                );
            }
            Err(_) => {}
        }
        packet_buffer.clear();
    }
}
