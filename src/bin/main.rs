#![no_std]
#![no_main]

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
use futures_util::future::{select, Either};

use esp_backtrace as _;

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

static INTERVAL_SIGNAL: Signal<CriticalSectionRawMutex, u32> = Signal::new();

#[embassy_executor::task]
async fn rx_task(mut rx: UsbSerialJtagRx<'static, esp_hal::Async>) {
    use embedded_io_async::Read;
    let mut buffer = [0u8; 32];
    let mut pos = 0;

    loop {
        let mut byte = [0u8; 1];
        // This is now TRULY async and won't block the other tasks
        if rx.read_exact(&mut byte).await.is_ok() {
            let b = byte[0];
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
    }
}

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

    // Initialize USB Serial in Async mode
    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE).into_async();
    let (rx, mut tx) = usb_serial.split();

    spawner.spawn(rx_task(rx)).ok();

    let temperature_sensor =
        TemperatureSensor::new(peripherals.TSENS, Config::default()).expect("Failed to init TSENS");

    let start_time = Instant::now();
    let mut buffer = [0u8; 32];
    let mut current_interval = 1000u32;

    loop {
        let temp = temperature_sensor.get_temperature();
        let data = Telemetry {
            temp: temp.to_celsius(),
            uptime_ms: start_time.elapsed().as_millis() as u32,
            interval_ms: current_interval,
        };

        if let Ok(bytes) = to_slice_cobs(&data, &mut buffer) {
            tx.write(bytes).ok();
        }

        // Wait for EITHER the timer OR a new command signal
        let timer_fut = Timer::after(Duration::from_millis(current_interval as u64));
        let signal_fut = INTERVAL_SIGNAL.wait();

        match select(timer_fut, signal_fut).await {
            Either::Left(_) => {
                // Timer finished normally, continue to next loop
            }
            Either::Right((new_millis, _)) => {
                // Command received! Update and restart loop immediately
                current_interval = new_millis;
                info!("Interval updated to {}ms", current_interval);
            }
        }
    }
}
