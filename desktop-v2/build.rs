fn main() {
    tauri_build::build();
    
    // Linux Mint specific build configurations
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=gtk-3");
        println!("cargo:rustc-link-lib=gdk-3");
        println!("cargo:rustc-link-lib=gdk_pixbuf-2.0");
        println!("cargo:rustc-link-lib=gio-2.0");
        println!("cargo:rustc-link-lib=gobject-2.0");
        println!("cargo:rustc-link-lib=glib-2.0");
    }
}
