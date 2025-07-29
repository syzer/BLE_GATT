# COA-GATT-bleps-c3 for ESP32-C3 with OLED

This project is for the ESP32-C3 microcontroller with an SSD1306 OLED display. It uses Rust and the Embassy async runtime.

Power draft is 0.32W baseline to 0.52W with the BLE transmission.

 _____
< BLE >
 -----
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||

## Build

```
cargo build
```

## Flash (replace <PORT> with your device port)

```
just 
# or 
cargo run
```

## Monitor Serial Output

```
just monitor
#or 
probe-rs attach --chip esp32c3 --probe 303a:1001:50:78:7D:B9:63:C0 target/riscv32imc-unknown-none-elf/debug/coa_gatt_bleps_c3
```

## Notes
- Target: ESP32-C3
- Display: SSD1306 OLED (I2C)
- Async: Embassy

## TODO 
- [ ] test with sdl2 https://crates.io/crates/embedded-graphics-simulator