#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::{info, warn};
use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::ble::controller::BleConnector;
use panic_rtt_target as _;
use ssd1306::I2CDisplayInterface;

use ssd1306::prelude::*;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::Ssd1306;

use trouble_host::prelude::ExternalController;

use esp_hal::tsens::{Config as TsensConfig, TemperatureSensor};

extern crate alloc;

use coa_gatt::mock::create_mock_display;
use coa_gatt::task::{ble, display_task, DisplayWrapper};
use coa_gatt::task::temp_task;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.5.0

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);
    // esp_alloc::heap_allocator!(size: 192 * 1024);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 64 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let wifi_init =
        esp_wifi::init(timer1.timer0, rng).expect("Failed to initialize WIFI/BLE controller");
    let (mut _wifi_controller, _interfaces) = esp_wifi::wifi::new(&wifi_init, peripherals.WIFI)
        .expect("Failed to initialize WIFI controller");
    let connector = BleConnector::new(&wifi_init, peripherals.BT);

    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .expect("Failed to create I2C instance")
    .with_sda(peripherals.GPIO5)
    .with_scl(peripherals.GPIO6);

    let interface = I2CDisplayInterface::new(i2c);
    let mut real_disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    // Try to initialize the real display, use mock display if it fails
    let display_wrapper = if real_disp.init().is_err() {
        warn!("Failed to initialize display, using mock display instead");
        DisplayWrapper::Mock(create_mock_display())
    } else {
        DisplayWrapper::Real(real_disp)
    };
    let controller: ExternalController<_, 20> = ExternalController::new(connector);
    let tsens = TemperatureSensor::new(peripherals.TSENS, TsensConfig::default())
        .expect("TSENS init failed");

    spawner.must_spawn(display_task(display_wrapper));
    spawner
        .must_spawn(temp_task(tsens));

    info!("Running BLE...");
    // TODO as task
    ble::run(controller).await;
}
