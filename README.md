# COA-GATT-bleps-c3 for ESP32-C3 with OLED

This project is for the ESP32-C3 microcontroller with an SSD1306 OLED display. It uses Rust and the Embassy async runtime.

Power draft is 0.32W baseline to 0.45W with the transmition on

## Build

```
cargo build
```

## Flash (replace <PORT> with your device port)

```
cargo run --release
# or use espflash if installed:
espflash /dev/<PORT> target/riscv32imc-unknown-none-elf/release/coa-gatt-bleps-c3
```

## Monitor Serial Output

```
cargo install espflash # if not already installed
espflash monitor /dev/<PORT>
```

## Notes
- Target: ESP32-C3
- Display: SSD1306 OLED (I2C)
- Async: Embassy

## TODO 
- [ ] test with sdl2 https://crates.io/crates/embedded-graphics-simulator