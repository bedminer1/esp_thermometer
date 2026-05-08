//! # Command Transmitter Module
//! 
//! Monitors user keyboard input and sends binary control packets to the Robot.

use common::Command;
use postcard::to_slice_cobs;
use std::io::{BufRead, Write};
use serialport::SerialPort;

/// Interactive shell that toggles robot settings on keypress.
/// 
/// Listens for a 'Newline' (Enter key) on stdin, encodes a `Command` enum,
/// and writes the binary frame to the serial port.
pub fn run_command_shell(mut port: Box<dyn SerialPort>) {
    let mut current_interval = 1000u32;
    println!("Controls: [Enter] to toggle 1s/5s logging frequency");

    let stdin = std::io::stdin();
    for _ in stdin.lock().lines() {
        // Toggle logic
        current_interval = if current_interval == 1000 { 5000 } else { 1000 };
        
        let cmd = Command::SetInterval { millis: current_interval };
        let mut buf = [0u8; 32];
        
        // Encode with COBS and send
        if let Ok(bytes) = to_slice_cobs(&cmd, &mut buf) {
            let _ = port.write_all(bytes);
            println!(">>> Sent Command: SetInterval {}ms", current_interval);
        }
    }
}
