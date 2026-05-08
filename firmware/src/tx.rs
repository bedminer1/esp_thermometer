//! # Telemetry Transmitter Module
//! 
//! Handles reading the internal temperature sensor and streaming binary
//! telemetry packets to the Station.

use common::Telemetry;
use crate::rx::INTERVAL_SIGNAL;
use embassy_time::{Duration, Instant, Timer};
use esp_hal::tsens::TemperatureSensor;
use esp_hal::usb_serial_jtag::UsbSerialJtagTx;
use futures_util::future::{select, Either};
use postcard::to_slice_cobs;

/// Embassy task that periodically reads sensors and sends telemetry.
/// 
/// It uses `select` to stay responsive to frequency changes sent by the Station.
#[embassy_executor::task]
pub async fn tx_task(
    mut tx: UsbSerialJtagTx<'static, esp_hal::Async>,
    tsens: TemperatureSensor<'static>,
) {
    let start_time = Instant::now();
    let mut buffer = [0u8; 32];
    let mut current_interval = 1000u32; // Default 1Hz

    loop {
        // 1. Collect and Format Data
        let temp = tsens.get_temperature();
        let data = Telemetry {
            temp: temp.to_celsius(),
            uptime_ms: start_time.elapsed().as_millis() as u32,
            interval_ms: current_interval,
        };

        // 2. Binary Serialization with COBS framing
        if let Ok(bytes) = to_slice_cobs(&data, &mut buffer) {
            let _ = tx.write(bytes);
        }

        // 3. Responsive Sleep
        // We wait for the timer OR the signal from the RX task.
        match select(
            Timer::after(Duration::from_millis(current_interval as u64)),
            INTERVAL_SIGNAL.wait(),
        )
        .await
        {
            Either::Left(_) => { /* Timer fired, repeat loop */ }
            Either::Right((new_millis, _)) => {
                // New interval received live! Update and restart loop immediately.
                current_interval = new_millis;
            }
        }
    }
}
