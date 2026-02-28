// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Force X11 backend to avoid Wayland protocol errors with
    // transparent windows and rdev's X11-based global input listener
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("GDK_BACKEND", "x11");
        // Fix transparent window rendering with WebKitGTK
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    bongo_nat_lib::run()
}
