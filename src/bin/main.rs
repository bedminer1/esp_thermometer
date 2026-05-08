#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::tsens::{Config, TemperatureSensor};
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_thermometer::Telemetry;
use postcard::to_slice_cobs;

use log::info;

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};

use esp_backtrace as _;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.2.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let _ = spawner;

    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let temperature_sensor =
        TemperatureSensor::new(peripherals.TSENS, Config::default()).expect("Failed to init TSENS");

    let start_time = Instant::now();
    let mut buffer = [0u8; 32];

    loop {
        let temp = temperature_sensor.get_temperature();
        
        let data = Telemetry {
            temp: temp.to_celsius(),
            uptime_ms: start_time.elapsed().as_millis() as u32,
        };

        // Serialize to bytes with COBS framing
        if let Ok(bytes) = to_slice_cobs(&data, &mut buffer) {
            usb_serial.write(bytes).ok();
        }

        Timer::after(Duration::from_millis(100)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples
}
