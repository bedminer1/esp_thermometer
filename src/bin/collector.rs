use serialport;
use std::time::Duration;
use esp_thermometer::Telemetry;
use postcard::from_bytes_cobs;
use std::io::Read;

fn main() {
    let port_name = "/dev/tty.usbmodem1401";
    let baud_rate = 115_200;

    println!("Opening serial port: {} at {} baud", port_name, baud_rate);

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(1000))
        .open()
        .expect("Failed to open port");

    let mut raw_buffer = [0u8; 1024];
    let mut packet_buffer = Vec::new();

    println!("Listening for telemetry... (Press Ctrl+C to stop)");

    loop {
        let mut byte = [0u8; 1];
        if port.read_exact(&mut byte).is_ok() {
            if byte[0] == 0 {
                // End of COBS frame detected
                if !packet_buffer.is_empty() {
                    // Try to decode
                    match from_bytes_cobs::<Telemetry>(&mut packet_buffer) {
                        Ok(data) => {
                            println!(
                                "[{}] Temp: {:.2}°C | Uptime: {}ms",
                                data.uptime_ms, data.temp, data.uptime_ms
                            );
                        }
                        Err(e) => {
                            eprintln!("Failed to decode packet: {:?}", e);
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
