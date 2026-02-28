use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // --- System Tray ---
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Bongo Nat")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            // --- Global Keyboard Listener (rdev) ---
            println!("[bongo-nat] Starting global keyboard listener...");
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                println!("[bongo-nat] Listener thread spawned");
                if let Err(e) = rdev::listen(move |event| {
                    if let rdev::EventType::KeyPress(key) = event.event_type {
                        println!("[bongo-nat] Key pressed: {:?}", key);
                        let _ = app_handle.emit("keypress", ());
                    }
                }) {
                    eprintln!("[bongo-nat] ERROR: Could not start global keyboard listener: {:?}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
