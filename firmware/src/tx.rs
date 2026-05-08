use common::Telemetry;
use crate::rx::INTERVAL_SIGNAL;
use embassy_time::{Duration, Instant, Timer};
use esp_hal::tsens::TemperatureSensor;
use esp_hal::usb_serial_jtag::UsbSerialJtagTx;
use futures_util::future::{select, Either};
use postcard::to_slice_cobs;

#[embassy_executor::task]
pub async fn tx_task(
    mut tx: UsbSerialJtagTx<'static, esp_hal::Async>,
    tsens: TemperatureSensor<'static>,
) {
    let start_time = Instant::now();
    let mut buffer = [0u8; 32];
    let mut current_interval = 1000u32;

    loop {
        let temp = tsens.get_temperature();
        let data = Telemetry {
            temp: temp.to_celsius(),
            uptime_ms: start_time.elapsed().as_millis() as u32,
            interval_ms: current_interval,
        };

        if let Ok(bytes) = to_slice_cobs(&data, &mut buffer) {
            tx.write(bytes).ok();
        }

        match select(
            Timer::after(Duration::from_millis(current_interval as u64)),
            INTERVAL_SIGNAL.wait(),
        )
        .await
        {
            Either::Left(_) => {}
            Either::Right((new_millis, _)) => {
                current_interval = new_millis;
            }
        }
    }
}
