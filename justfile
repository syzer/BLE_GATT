# Use ash shell for all recipes
set shell := ["bash", "-c"]

# List available debug probes
default:
    cargo run

# Alias for listing probes
list-probes:
    probe-rs list

# Automatically attach to the first detected ESP JTAG probe and filter INFO lines
monitor:
    @probe_id=$(just list-probes | grep '\[0\]' | sed -E 's/.* -- ([^ ]+) .*/\1/'); \
    probe-rs attach --chip esp32c6 --probe $probe_id \
      target/riscv32imc-unknown-none-elf/debug/coa_gatt_bleps_c3 | grep INFO