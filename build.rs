//! Provides the target-appropriate linker script (`memory.x`) to
//! cortex-m-rt's `link.x`: RP2350 (real hardware) by default, or RP2040
//! (Wokwi simulation stand-in) when built with `--features rp2040`.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    let memory_x: &[u8] = if std::env::var_os("CARGO_FEATURE_RP2040").is_some() {
        include_bytes!("memory_rp2040.x")
    } else {
        include_bytes!("memory.x")
    };
    let mut f = File::create(out.join("memory.x")).unwrap();
    f.write_all(memory_x).unwrap();
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=memory_rp2040.x");
    println!("cargo:rerun-if-changed=build.rs");
}
