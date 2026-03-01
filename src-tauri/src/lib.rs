use std::collections::HashSet;
use std::sync::Mutex;

use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // --- System Tray ---
            let github = MenuItem::with_id(app, "github", "Visit GitHub page", true, None::<&str>)?;
            let show_bg = CheckMenuItem::with_id(app, "show_bg", "Show Background", true, false, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_bg, &github, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Bongo Nat")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    } else if event.id().as_ref() == "show_bg" {
                        let _ = app.emit("toggle-background", ());
                    } else if event.id().as_ref() == "github" {
                        let _ = tauri_plugin_opener::open_url("https://github.com/pontusasp/bongo-nat", None::<&str>);
                    }
                })
                .build(app)?;

            // --- Global Keyboard Listener (rdev) ---
            println!("[bongo-nat] Starting global keyboard listener...");
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                println!("[bongo-nat] Listener thread spawned");
                let held_keys: Mutex<HashSet<rdev::Key>> = Mutex::new(HashSet::new());
                let held_buttons: Mutex<HashSet<rdev::Button>> = Mutex::new(HashSet::new());
                if let Err(e) = rdev::listen(move |event| {
                    match event.event_type {
                        rdev::EventType::KeyPress(key) => {
                            let mut keys = held_keys.lock().unwrap();
                            if keys.insert(key) {
                                // First press – not a repeat
                                println!("[bongo-nat] Key pressed: {:?}", key);
                                let _ = app_handle.emit("keypress", ());
                            }
                        }
                        rdev::EventType::KeyRelease(key) => {
                            held_keys.lock().unwrap().remove(&key);
                        }
                        rdev::EventType::ButtonPress(button) => {
                            let mut buttons = held_buttons.lock().unwrap();
                            if buttons.insert(button) {
                                println!("[bongo-nat] Mouse button pressed: {:?}", button);
                                let _ = app_handle.emit("keypress", ());
                            }
                        }
                        rdev::EventType::ButtonRelease(button) => {
                            held_buttons.lock().unwrap().remove(&button);
                        }
                        _ => {}
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
