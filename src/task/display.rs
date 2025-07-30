use defmt::{info, warn};
use embassy_executor::task;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::FONT_6X9;
use embedded_graphics::mono_font::{MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;

use crate::display::update_display;
use crate::mock::MockDisplayType;

// Import the DisplayType from main
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::I2CInterface;
use ssd1306::size::DisplaySize128x64;
use ssd1306::Ssd1306;
use esp_hal::i2c::master::I2c;
use esp_hal::Blocking;

// Define the concrete display type
pub type DisplayType = Ssd1306<
    I2CInterface<I2c<'static, Blocking>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

// Define a display wrapper that can work with the specific display type
pub enum DisplayWrapper {
    Real(DisplayType),
    Mock(MockDisplayType),
}

#[task]
pub async fn display_task(
    mut disp: DisplayWrapper,
) {
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(BinaryColor::On)
        .build();

    let mut counter = 0;
    let x_offset = 30;
    let y_offset = 22;

    loop {
        // Update the display using the common function
        match &mut disp {
            DisplayWrapper::Real(real_disp) => {
                if update_display(real_disp, counter, x_offset, y_offset, text_style).is_err() {
                    warn!("Error updating real display");
                    continue;
                }

                if real_disp.flush().is_err() {
                    warn!("Failed to flush real display");
                    continue;
                }
            },
            DisplayWrapper::Mock(mock_disp) => {
                if update_display(mock_disp, counter, x_offset, y_offset, text_style).is_err() {
                    warn!("Error updating mock display");
                    continue;
                }

                if mock_disp.flush().is_err() {
                    warn!("Failed to flush mock display");
                    continue;
                }
            }
        }

        info!("Display updated with counter: {}", counter);
        counter += 1;
        Timer::after(Duration::from_secs(1)).await;
    }
}
