use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {

    //#[cfg(not(feature = "cortexm"))]
    let memory_x = ("memory.x", include_bytes!("memory.x"));
    /*#[cfg(feature = "qemu")]
    let memory_x = ("memory-qemu.x", include_bytes!("memory-qemu.x"));
    #[cfg(feature = "stm32f4")]
    let memory_x = ("memory-stm32f4.x", include_bytes!("memory-stm32f4.x"));
    #[cfg(feature = "atsame54")]
    let memory_x = ("memory-atsame54.x", include_bytes!("memory-atsame54.x"));
    #[cfg(feature = "daisy")]
    let memory_x = ("memory-daisy.x", include_bytes!("memory-daisy.x"));*/

    if env::var_os("CARGO_FEATURE_RT").is_some() {
        let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
        File::create(out.join(memory_x.0))
            .unwrap()
            .write_all(memory_x.1)
            .unwrap();
        println!("cargo:rustc-link-search={}", out.display());
        println!("cargo:rerun-if-changed={}", memory_x.0);
    }

    println!("cargo:rerun-if-changed=build.rs");
}
