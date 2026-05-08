//! # Station Orchestrator (main.rs)
//! 
//! The entry point for the Mac/Host CLI application.
//! Sets up the serial port and manages the threading for simultaneous RX/TX.

use serialport;
use std::time::Duration;
use station::rx::run_telemetry_loop;
use station::tx::run_command_shell;

/// CLI Entry Point.
/// 
/// 1. Connects to the ESP32-C6 via its USB-Serial port.
/// 2. Clones the port handle to allow bidirectional communication.
/// 3. Spawns a dedicated thread for user commands.
/// 4. Runs the telemetry receiver on the main thread.
fn main() {
    let port_name = "/dev/tty.usbmodem1401"; // Standard ESP32-C6 port name on macOS
    let baud_rate = 115_200;

    println!("Opening serial port: {} at {} baud", port_name, baud_rate);

    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10)) // Short timeout to keep loops responsive
        .open()
        .expect("Failed to open port. Is the ESP32 plugged in?");

    // Create a separate handle for sending data
    let port_tx = port.try_clone().expect("Failed to clone port for TX");
    let port_rx = port; // Original handle used for receiving

    // Start the keyboard command transmitter in the background
    std::thread::spawn(move || {
        run_command_shell(port_tx);
    });

    // Start receiving and decoding telemetry in the foreground
    run_telemetry_loop(port_rx);
}
