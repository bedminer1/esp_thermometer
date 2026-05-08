use serialport;
use std::time::Duration;
use serde::Deserialize;
use postcard::from_bytes_cobs;
use std::io::Read;

#[derive(Deserialize, Debug)]
pub struct Telemetry {
    pub temp: f32,
    pub uptime_ms: u32,
}

fn main() {
    let port_name = "/dev/tty.usbmodem1401";
    let baud_rate = 115_200;

    println!("Opening serial port: {} at {} baud", port_name, baud_rate);

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(1000))
        .open()
        .expect("Failed to open port");

    let mut packet_buffer = Vec::new();

    println!("Listening for telemetry... (Press Ctrl+C to stop)");

    loop {
        let mut byte = [0u8; 1];
        if port.read_exact(&mut byte).is_ok() {
            if byte[0] == 0 {
                if !packet_buffer.is_empty() {
                    match from_bytes_cobs::<Telemetry>(&mut packet_buffer) {
                        Ok(data) => {
                            println!(
                                "[Uptime: {:5}ms] Internal Temp: {:.2}°C",
                                data.uptime_ms, data.temp
                            );
                        }
                        Err(_) => {
                            // Ignore incomplete/corrupt packets during sync
                        }
                    }
                    packet_buffer.clear();
                }
            } else {
                packet_buffer.push(byte[0]);
            }
        }
    }
}
