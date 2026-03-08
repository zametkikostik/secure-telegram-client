//! Build script for liberty-reach-core
//! 
//! Генерирует FFI bindings для Flutter и других платформ.

use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    
    // Генерация заголовочных файлов для C/C++
    #[cfg(feature = "ffi")]
    generate_c_headers();
    
    // Генерация bindings для Flutter
    #[cfg(feature = "flutter")]
    generate_flutter_bindings();
}

#[cfg(feature = "ffi")]
fn generate_c_headers() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    let config = cbindgen::Config {
        header: Some("/* Liberty Reach Messenger Core - C Bindings */".to_string()),
        include_guard: Some("LIBERTY_REACH_CORE_H".to_string()),
        language: cbindgen::Language::C,
        style: cbindgen::Style::Both,
        ..Default::default()
    };
    
    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate C bindings")
        .write_to_file("include/liberty_reach_core.h");
    
    println!("cargo:warning=Generated C bindings");
}

#[cfg(feature = "flutter")]
fn generate_flutter_bindings() {
    // Flutter bindings генерируются через flutter_rust_bridge
    println!("cargo:warning=Flutter bindings generation not implemented in build.rs");
    println!("cargo:warning=Use: cargo run -p flutter_rust_bridge -- generate");
}
