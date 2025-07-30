#![no_std]

extern crate alloc;

pub mod task;

pub mod mock {
    use alloc::vec;
    use alloc::vec::Vec;
    use core::cell::RefCell;
    use embedded_graphics::{geometry::OriginDimensions, pixelcolor::BinaryColor, prelude::*};
    // No imports needed from ssd1306

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

    impl OriginDimensions for MockDisplay {
        fn size(&self) -> Size {
            Size::new(self.width, self.height)
        }
    }

    impl DrawTarget for MockDisplay {
        type Color = BinaryColor;
        type Error = ();

        fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            // In a real implementation, this would update the buffer with the pixels
            // For the mock, we just return success
            Ok(())
        }

        fn clear(&mut self, _color: Self::Color) -> Result<(), Self::Error> {
            // Clear the buffer manually instead of calling self.clear()
            let mut buffer = self.buffer.borrow_mut();
            for byte in buffer.iter_mut() {
                *byte = 0;
            }
            Ok(())
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
    use alloc::format;
    use alloc::string::String;
    use embedded_graphics::{
        mono_font::MonoTextStyle, pixelcolor::BinaryColor, prelude::*, text::Text,
    };

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

        // Determine which version of the speech bubble to display based on counter % 5
        let (first_line, second_line, third_line) = match counter % 5 {
            1 => ("B", "L", "E"), // 1st iteration: "BLE"
            2 => (" ", " ", " "), // 2nd iteration: "" (empty)
            3 => ("B", " ", " "), // 3rd iteration: "B"
            4 => ("B", "L", " "), // 4th iteration: "BL"
            0 => ("B", "L", "E"), // 5th iteration: "BLE" (same as 1st)
            _ => unreachable!(),
        };

        let cow_art = [
            r"       ",
            r"  ^__^",
            &format!("{} (oo)\\____", first_line),
            &format!("{} (__)\\       )\\/\\", second_line),
            &format!("{}     ||--w ||", third_line),
            r"      ||       ||",
        ];

        // Draw the cow ASCII art with smaller vertical spacing
        for (i, line) in cow_art.iter().enumerate() {
            // If this is the line with the eyes (index 2) and counter is divisible by 10,
            // replace "(oo)" with "(X.)"
            let display_line = if i == 2 && counter % 10 == 0 {
                "B (X.)\\____"
            } else {
                line
            };

            Text::new(
                display_line,
                Point::new(x_offset, y_offset + (i as i32 * 7)),
                text_style,
            )
            .draw(display)?;
        }

        // Create a string with the counter value at the bottom
        let counter_text: String = format!("Moo: {}s", counter);
        Text::new(
            &counter_text,
            Point::new(x_offset, y_offset + 64),
            text_style,
        )
        .draw(display)?;

        Ok(())
    }
}
