#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::tsens::{Config, TemperatureSensor};
use esp_hal::usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagRx};
use esp_thermometer::{Command, Telemetry};
use postcard::{from_bytes_cobs, to_slice_cobs};

use log::info;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};

use esp_backtrace as _;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

static INTERVAL_SIGNAL: Signal<CriticalSectionRawMutex, u32> = Signal::new();

#[embassy_executor::task]
async fn rx_task(rx: &'static mut UsbSerialJtagRx<'static, esp_hal::Blocking>) {
    let mut buffer = [0u8; 32];
    let mut pos = 0;

    loop {
        match rx.read_byte() {
            Ok(b) => {
                if b == 0 {
                    if pos > 0 {
                        if let Ok(cmd) = from_bytes_cobs::<Command>(&mut buffer[..pos]) {
                            match cmd {
                                Command::SetInterval { millis } => {
                                    INTERVAL_SIGNAL.signal(millis);
                                }
                                Command::ToggleInterval => {}
                            }
                        }
                        pos = 0;
                    }
                } else if pos < buffer.len() {
                    buffer[pos] = b;
                    pos += 1;
                }
            }
            Err(_) => {
                // Yield if no data
                Timer::after(Duration::from_millis(10)).await;
            }
        }
    }
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
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

    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let (rx, mut tx) = usb_serial.split();

    static RX_CELL: static_cell::StaticCell<UsbSerialJtagRx<'static, esp_hal::Blocking>> =
        static_cell::StaticCell::new();
    let rx = RX_CELL.init(rx);

    spawner.spawn(rx_task(rx)).ok();

    let temperature_sensor =
        TemperatureSensor::new(peripherals.TSENS, Config::default()).expect("Failed to init TSENS");

    let start_time = Instant::now();
    let mut buffer = [0u8; 32];
    let mut current_interval = 1000u32;

    loop {
        if INTERVAL_SIGNAL.signaled() {
            current_interval = INTERVAL_SIGNAL.wait().await;
            info!("Interval changed to {}ms", current_interval);
        }

        let temp = temperature_sensor.get_temperature();
        
        let data = Telemetry {
            temp: temp.to_celsius(),
            uptime_ms: start_time.elapsed().as_millis() as u32,
            interval_ms: current_interval,
        };

        if let Ok(bytes) = to_slice_cobs(&data, &mut buffer) {
            tx.write(bytes).ok();
        }

        Timer::after(Duration::from_millis(current_interval as u64)).await;
    }
}
