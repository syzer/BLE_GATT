use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let _ = dotenvy::from_filename(".env");
    println!("cargo:rerun-if-changed=.env");

    // Get MAC address from environment variable
    let mac_str = std::env::var("MAC").unwrap_or_else(|_| "ff:8f:1a:05:e4:ff".to_string());

    // Parse MAC address into bytes
    let mac_bytes: Vec<u8> = mac_str
        .split(':')
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();

    // Generate Rust code with MAC address as a constant
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("mac_address.rs");
    let mut f = File::create(&dest_path).unwrap();

    if mac_bytes.len() == 6 {
        writeln!(f, "pub const MAC_ADDRESS: [u8; 6] = [{:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}];", 
            mac_bytes[0], mac_bytes[1], mac_bytes[2], mac_bytes[3], mac_bytes[4], mac_bytes[5]).unwrap();
    } else {
        // Fallback to default if MAC format is invalid
        writeln!(f, "pub const MAC_ADDRESS: [u8; 6] = [0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff];").unwrap();
    }

    linker_be_nice();
    println!("cargo:rustc-link-arg-tests=-Tembedded-test.x");
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    // make sure linkall.x is the last linker script (otherwise might cause problems with flip-link)
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                "_defmt_timestamp" => {
                    eprintln!();
                    eprintln!("💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`");
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                "esp_wifi_preempt_enable"
                | "esp_wifi_preempt_yield_task"
                | "esp_wifi_preempt_task_create" => {
                    eprintln!();
                    eprintln!("💡 `esp-wifi` has no scheduler enabled. Make sure you have the `builtin-scheduler` feature enabled, or that you provide an external scheduler.");
                    eprintln!();
                }
                "embedded_test_linker_file_not_added_to_rustflags" => {
                    eprintln!();
                    eprintln!("💡 `embedded-test` not found - make sure `embedded-test.x` is added as a linker script for tests");
                    eprintln!();
                }
                _ => (),
            },
            // we don't have anything helpful for "missing-lib" yet
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}
