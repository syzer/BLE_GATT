#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::{Spawner, task};
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::ble::controller::BleConnector;
use panic_rtt_target as _;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use ssd1306::I2CDisplayInterface;

use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use embedded_graphics::mono_font::ascii::FONT_6X9;
use ssd1306::prelude::*;
use ssd1306::Ssd1306;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x64;

extern crate alloc;

use alloc::format;
use alloc::string::String;
use esp_hal::Blocking;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::I2CInterface;

// Define a type alias for the display type
type DisplayType = Ssd1306<I2CInterface<I2c<'static, Blocking>>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[task]
async fn display_task(
    mut disp: DisplayType,
    text_style: MonoTextStyle<'static, BinaryColor>
) {
    let mut counter = 0;
    let x_offset = 30;
    let y_offset = 22;

    loop {

        // Clear display first
        disp.clear(BinaryColor::Off).unwrap();

        // Draw the three lines of text with offsets
        Text::new("COA", Point::new(x_offset, y_offset + 10), text_style)
            .draw(&mut disp)
            .unwrap();
        Text::new("BLE", Point::new(x_offset, y_offset + 20), text_style)
            .draw(&mut disp)
            .unwrap();

        // Create a string with the counter value
        let counter_text: String = format!("Running: {}s", counter);

        Text::new(&counter_text, Point::new(x_offset, y_offset + 30), text_style)
            .draw(&mut disp)
            .unwrap();

        // Update the display
        disp.flush().unwrap();

        info!("Display updated with counter: {}", counter);
        counter += 1;
        Timer::after(Duration::from_secs(1)).await;
    }
}


#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.5.0

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);
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
    let _connector = BleConnector::new(&wifi_init, peripherals.BT);

    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
        .unwrap()
        .with_sda(peripherals.GPIO5)
        .with_scl(peripherals.GPIO6);

    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    disp.init().unwrap();


    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(BinaryColor::On)
        .build();

    // TODO: Spawn some tasks
    let _ = spawner;
    spawner.spawn(display_task(
        disp,
        text_style,
    )).unwrap();

    loop {
        Timer::after(Duration::from_secs(60)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
