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

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE).into_async();
    let (rx, tx) = usb_serial.split();

    let tsens = TemperatureSensor::new(peripherals.TSENS, Config::default()).expect("Failed to init TSENS");

    spawner.spawn(rx_task(rx)).expect("Failed to spawn rx_task");
    spawner.spawn(tx_task(tx, tsens)).expect("Failed to spawn tx_task");

    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}
