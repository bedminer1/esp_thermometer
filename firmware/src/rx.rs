//! # Command Receiver Module
//! 
//! Handles the reception and decoding of binary commands from the USB-Serial port.
//! Incoming data is decoded using COBS framing and Postcard.

use common::Command;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embedded_io_async::Read;
use esp_hal::usb_serial_jtag::UsbSerialJtagRx;
use postcard::from_bytes_cobs;

/// Shared signal used to communicate interval changes to the TX task.
pub static INTERVAL_SIGNAL: Signal<CriticalSectionRawMutex, u32> = Signal::new();

/// Embassy task that constantly monitors the serial RX pipe.
/// 
/// It accumulates bytes into a buffer until a '0' (COBS delimiter) is found,
/// then attempts to decode the packet into a `Command`.
#[embassy_executor::task]
pub async fn rx_task(mut rx: UsbSerialJtagRx<'static, esp_hal::Async>) {
    let mut buffer = [0u8; 32];
    let mut pos = 0;

    loop {
        let mut byte = [0u8; 1];
        // Asynchronously wait for a single byte to arrive
        if rx.read_exact(&mut byte).await.is_err() {
            continue;
        }

        let b = byte[0];
        
        // Accumulate data bytes
        if b != 0 {
            if pos < buffer.len() {
                buffer[pos] = b;
                pos += 1;
            }
            continue;
        }

        // COBS Delimiter found: Process the packet
        if pos == 0 {
            continue;
        }

        if let Ok(cmd) = from_bytes_cobs::<Command>(&mut buffer[..pos]) {
            match cmd {
                Command::SetInterval { millis } => {
                    // Send the new value to the TX task via the signal
                    INTERVAL_SIGNAL.signal(millis);
                }
                _ => {}
            }
        }
        pos = 0;
    }
}
