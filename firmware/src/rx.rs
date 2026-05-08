use common::Command;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embedded_io_async::Read;
use esp_hal::usb_serial_jtag::UsbSerialJtagRx;
use postcard::from_bytes_cobs;

pub static INTERVAL_SIGNAL: Signal<CriticalSectionRawMutex, u32> = Signal::new();

#[embassy_executor::task]
pub async fn rx_task(mut rx: UsbSerialJtagRx<'static, esp_hal::Async>) {
    let mut buffer = [0u8; 32];
    let mut pos = 0;

    loop {
        let mut byte = [0u8; 1];
        if rx.read_exact(&mut byte).await.is_err() {
            continue;
        }

        let b = byte[0];
        if b != 0 {
            if pos < buffer.len() {
                buffer[pos] = b;
                pos += 1;
            }
            continue;
        }

        if pos == 0 {
            continue;
        }

        if let Ok(cmd) = from_bytes_cobs::<Command>(&mut buffer[..pos]) {
            match cmd {
                Command::SetInterval { millis } => INTERVAL_SIGNAL.signal(millis),
                _ => {}
            }
        }
        pos = 0;
    }
}
