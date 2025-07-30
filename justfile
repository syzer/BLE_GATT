# Use ash shell for all recipes
set shell := ["bash", "-c"]

# List available debug probes
default:
    just run-c3

# Run on ESP32-C3
run-c3:
    cargo build --features esp32c3 --target riscv32imc-unknown-none-elf
    probe-rs run --chip=esp32c3 --preverify --always-print-stacktrace --no-location --catch-hardfault --connect-under-reset target/riscv32imc-unknown-none-elf/debug/coa_gatt

# Run on ESP32-C6
run-c6:
    cargo build --no-default-features --features esp32c6 --target riscv32imac-unknown-none-elf
    probe-rs run --chip=esp32c6 --preverify --always-print-stacktrace --no-location --catch-hardfault target/riscv32imac-unknown-none-elf/debug/coa_gatt

# Alias for listing probes
list-probes:
    probe-rs list

# Automatically attach to the first detected ESP JTAG probe and filter INFO lines
monitor-c3:
    @probe_id=$(just list-probes | grep '\[0\]' | sed -E 's/.* -- ([^ ]+) .*/\1/'); \
    probe-rs attach --chip esp32c3 --probe $probe_id --connect-under-reset \
      target/riscv32imc-unknown-none-elf/debug/coa_gatt | grep INFO

monitor-c6:
    @probe_id=$(just list-probes | grep '\[0\]' | sed -E 's/.* -- ([^ ]+) .*/\1/'); \
    probe-rs attach --chip esp32c6 --probe $probe_id \
      target/riscv32imac-unknown-none-elf/debug/coa_gatt | grep INFO

# For backward compatibility
monitor: monitor-c3

# Monitor release build for ESP32-C3 with defmt-print
monitor-release-c3:
    probe-rs run --chip esp32c3 --connect-under-reset \
      target/riscv32imc-unknown-none-elf/release/coa_gatt | \
      defmt-print -e target/riscv32imc-unknown-none-elf/release/coa_gatt

# Monitor release build for ESP32-C6 with defmt-print
monitor-release-c6:
    probe-rs run --chip esp32c6 --connect-under-reset \
      target/riscv32imac-unknown-none-elf/release/coa_gatt | \
      defmt-print -e target/riscv32imac-unknown-none-elf/release/coa_gatt

# For backward compatibility
monitor-release: monitor-release-c3

reset-c3:
    @probe_id=$(just list-probes | grep '\[0\]' | sed -E 's/.* -- ([^ ]+) .*/\1/'); \
    pkill probe-rs; \
    probe-rs reset --chip esp32c3 --probe $probe_id --connect-under-reset

reset-c6:
    @probe_id=$(just list-probes | grep '\[0\]' | sed -E 's/.* -- ([^ ]+) .*/\1/'); \
    pkill probe-rs; \
    probe-rs reset --chip esp32c6 --probe $probe_id

# For backward compatibility
reset: reset-c3

# Build, flash, and run release version for ESP32-C3
release-c3:
    cargo build --release --features esp32c3 --target riscv32imc-unknown-none-elf
    probe-rs run --chip=esp32c3 --preverify --always-print-stacktrace --no-location --catch-hardfault --connect-under-reset target/riscv32imc-unknown-none-elf/release/coa_gatt

# Build, flash, and run release version for ESP32-C6
release-c6:
    cargo build --release --no-default-features --features esp32c6 --target riscv32imac-unknown-none-elf
    probe-rs run --chip=esp32c6 --preverify --always-print-stacktrace --no-location --catch-hardfault target/riscv32imac-unknown-none-elf/release/coa_gatt

# Estimate flash usage of the debug build
#flash-size:
#    @cargo build
#    @riscv32-unknown-elf-size \
#      target/riscv32imc-unknown-none-elf/debug/coa_gatt
