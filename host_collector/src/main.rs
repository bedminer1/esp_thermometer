use serialport;
use std::time::Duration;
use serde::Deserialize;
use postcard::from_bytes_cobs;
use std::io::Read;

#[derive(Deserialize, Debug)]
pub struct Telemetry {
    pub temp: f32,
    pub uptime_ms: u32,
    pub interval_ms: u32,
}

#[derive(serde::Serialize, Debug)]
pub enum Command {
    SetInterval { millis: u32 },
    ToggleInterval,
}

fn main() {
    let port_name = "/dev/tty.usbmodem1401";
    let baud_rate = 115_200;

    println!("Opening serial port: {} at {} baud", port_name, baud_rate);

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    let mut clone = port.try_clone().expect("Failed to clone port");
    
    // Spawn a thread to handle keyboard input
    std::thread::spawn(move || {
        let mut current_interval = 1000u32;
        println!("Controls: [Space] to toggle 1s/5s logging");
        
        // This is a bit rough for a CLI but works for a quick test
        use std::io::BufRead;
        let stdin = std::io::stdin();
        for _ in stdin.lock().lines() {
            current_interval = if current_interval == 1000 { 5000 } else { 1000 };
            let cmd = Command::SetInterval { millis: current_interval };
            let mut buf = [0u8; 32];
            if let Ok(bytes) = postcard::to_slice_cobs(&cmd, &mut buf) {
                clone.write_all(bytes).ok();
                println!(">>> Sent Command: SetInterval {}ms", current_interval);
            }
        }
    });

    let mut packet_buffer = Vec::new();
    println!("Listening for telemetry... (Press Enter to toggle frequency)");

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
