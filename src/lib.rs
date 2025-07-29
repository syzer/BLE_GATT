#![no_std]

extern crate alloc;

#[cfg(test)]
pub mod mock {
    use alloc::vec::Vec;
    use core::cell::RefCell;
    use embedded_graphics::{
        pixelcolor::BinaryColor,
        prelude::*,
    };
    use ssd1306::{
        mode::BufferedGraphicsMode,
        prelude::*,
        size::DisplaySize128x64,
    };

    // Mock implementation of the display
    pub struct MockDisplay {
        buffer: RefCell<Vec<u8>>,
        width: u32,
        height: u32,
    }

    impl MockDisplay {
        pub fn new() -> Self {
            Self {
                buffer: RefCell::new(vec![0; (128 * 64 / 8) as usize]),
                width: 128,
                height: 64,
            }
        }

        pub fn clear(&self) -> Result<(), ()> {
            let mut buffer = self.buffer.borrow_mut();
            for byte in buffer.iter_mut() {
                *byte = 0;
            }
            Ok(())
        }

        pub fn flush(&self) -> Result<(), ()> {
            // In a real implementation, this would send the buffer to the display
            // For the mock, we just return success
            Ok(())
        }
    }

    impl DrawTarget for MockDisplay {
        type Color = BinaryColor;
        type Error = ();

        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            // In a real implementation, this would update the buffer with the pixels
            // For the mock, we just return success
            Ok(())
        }

        fn clear(&mut self, _color: Self::Color) -> Result<(), Self::Error> {
            self.clear()
        }

        fn dimensions(&self) -> Size {
            Size::new(self.width, self.height)
        }
    }

    // Type alias for the mock display that matches the real display type
    pub type MockDisplayType = MockDisplay;

    // Function to create a mock display for testing
    pub fn create_mock_display() -> MockDisplayType {
        MockDisplay::new()
    }
}

// Re-export the display task function for both main and testing
pub mod display {
    use embedded_graphics::{
        mono_font::MonoTextStyle,
        pixelcolor::BinaryColor,
        prelude::*,
        text::Text,
    };
    use alloc::format;
    use alloc::string::String;

    // Function to update the display with the given counter value
    pub fn update_display<D>(
        display: &mut D,
        counter: u32,
        x_offset: i32,
        y_offset: i32,
        text_style: MonoTextStyle<'_, BinaryColor>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        // Clear display first
        display.clear(BinaryColor::Off)?;

        // Draw the three lines of text with offsets
        Text::new("COA", Point::new(x_offset, y_offset + 10), text_style)
            .draw(display)?;
        Text::new("BLE", Point::new(x_offset, y_offset + 20), text_style)
            .draw(display)?;

        // Create a string with the counter value
        let counter_text: String = format!("Running: {}s", counter);

        Text::new(&counter_text, Point::new(x_offset, y_offset + 30), text_style)
            .draw(display)?;

        Ok(())
    }
}
