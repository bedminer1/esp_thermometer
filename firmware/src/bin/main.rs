//! # Firmware Orchestrator (main.rs)
//! 
//! The entry point for the ESP32-C6 firmware. Its sole responsibility is to
//! initialize the silicon hardware, set up the Embassy executor, and spawn
//! the RX and TX modules.

#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::tsens::{Config, TemperatureSensor};
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_thermometer::rx::rx_task;
use esp_thermometer::tx::tx_task;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use esp_backtrace as _;

extern crate alloc;

// Hardware descriptor required for the bootloader
esp_bootloader_esp_idf::esp_app_desc!();

/// System entry point.
/// 
/// 1. Initializes system clocks and peripherals.
/// 2. Sets up the heap for the radio stack.
/// 3. Initializes async serial and internal sensors.
/// 4. Spawns the background telemetry and command tasks.
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // Basic init
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize 64KB heap for radio/async operations
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    // Start system timer for Embassy
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // Initialize USB-Serial driver in Async mode
    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE).into_async();
    let (rx, tx) = usb_serial.split();

    // Initialize internal temperature sensor
    let tsens = TemperatureSensor::new(peripherals.TSENS, Config::default()).expect("Failed to init TSENS");

    // Hand off hardware handles to the background tasks
    spawner.spawn(rx_task(rx)).expect("Failed to spawn rx_task");
    spawner.spawn(tx_task(tx, tsens)).expect("Failed to spawn tx_task");

    loop {
        // The main task has nothing to do but stay alive
        Timer::after(Duration::from_secs(60)).await;
    }
}
