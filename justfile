# List available debug probes
default:
    cargo run

# Alias for listing probes
list-probes:
    probe-rs list

# Attach to a probe (replace <probe> with the actual probe name or index)
attach probe:
    probe-rs attach --probe "{{probe}}"