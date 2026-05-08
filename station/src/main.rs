use serialport;
use std::time::Duration;
use station::rx::run_telemetry_loop;
use station::tx::run_command_shell;

fn main() {
    let port_name = "/dev/tty.usbmodem1401";
    let baud_rate = 115_200;

    println!("Opening serial port: {} at {} baud", port_name, baud_rate);

    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    let port_tx = port.try_clone().expect("Failed to clone port for TX");
    let port_rx = port;

    std::thread::spawn(move || {
        run_command_shell(port_tx);
    });

    run_telemetry_loop(port_rx);
}
