use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A discovered bongo version (subdirectory with idle.png, left.png, right.png).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BongoVersion {
    /// Display name (directory name).
    pub name: String,
    /// Absolute path to the version directory, or `None` for the built-in default.
    pub path: Option<String>,
}

/// Persisted user config.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Config {
    /// `None` means the built-in default.
    selected_version: Option<String>,
}

const DEFAULT_VERSION_NAME: &str = "Bongo Nat (Default)";
const VERSIONS_DIR_NAME: &str = "bongo-versions";
const CONFIG_FILE_NAME: &str = "bongo-nat-config.json";

// ---------------------------------------------------------------------------
// Helpers – filesystem
// ---------------------------------------------------------------------------

/// Return (and create if missing) the `bongo-versions` directory inside
/// the platform app-data dir.
fn versions_dir(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join(VERSIONS_DIR_NAME);
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
        // Write a helpful instructions file on first creation
        let instructions = "\
How to add a custom Bongo Cat version
======================================

1. Create a new folder inside this directory.
   The folder name will appear in the tray menu
   (e.g. \"My Cool Cat\").

2. Place exactly three PNG images in that folder:
     - idle.png   (the resting pose)
     - left.png   (left-paw bonk)
     - right.png  (right-paw bonk)

3. Restart Bongo Nat.

4. Right-click the tray icon and open the
   \"Bongo Version\" submenu to select your new version.

Note: Changes to this folder are only picked up when
the application starts. Please restart Bongo Nat after
adding, removing, or renaming version folders.
";
        let _ = fs::write(dir.join("instructions.txt"), instructions);
    }
    dir
}

/// Scan `bongo-versions/` for valid version subdirectories.
fn scan_versions(dir: &PathBuf) -> Vec<BongoVersion> {
    let mut versions = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return versions,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let has_idle = path.join("idle.png").is_file();
        let has_left = path.join("left.png").is_file();
        let has_right = path.join("right.png").is_file();
        if has_idle && has_left && has_right {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                versions.push(BongoVersion {
                    name: name.to_string(),
                    path: Some(path.to_string_lossy().to_string()),
                });
            }
        }
    }
    versions.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    versions
}

// ---------------------------------------------------------------------------
// Helpers – config persistence
// ---------------------------------------------------------------------------

fn config_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join(CONFIG_FILE_NAME)
}

fn load_config(app: &AppHandle) -> Config {
    let path = config_path(app);
    match fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

fn save_config(app: &AppHandle, config: &Config) {
    let path = config_path(app);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(config) {
        let _ = fs::write(&path, data);
    }
}

// ---------------------------------------------------------------------------
// Helpers – resolve the active BongoVersion
// ---------------------------------------------------------------------------

fn resolve_current_version(config: &Config, versions: &[BongoVersion]) -> BongoVersion {
    if let Some(ref sel) = config.selected_version {
        if let Some(v) = versions.iter().find(|v| v.name == *sel) {
            return v.clone();
        }
    }
    // Fallback to built-in default
    BongoVersion {
        name: DEFAULT_VERSION_NAME.to_string(),
        path: None,
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_versions(app: AppHandle) -> Vec<BongoVersion> {
    let dir = versions_dir(&app);
    scan_versions(&dir)
}

#[tauri::command]
fn get_current_version(app: AppHandle) -> BongoVersion {
    let config = load_config(&app);
    let dir = versions_dir(&app);
    let versions = scan_versions(&dir);
    resolve_current_version(&config, &versions)
}

#[tauri::command]
fn get_versions_dir(app: AppHandle) -> String {
    versions_dir(&app).to_string_lossy().to_string()
}

/// Read a version image and return it as a data: URL (base64-encoded PNG).
/// `image_name` must be one of "idle", "left", "right".
#[tauri::command]
fn get_version_image(
    app: AppHandle,
    version_name: String,
    image_name: String,
) -> Result<String, String> {
    let dir = versions_dir(&app);
    let path = dir.join(&version_name).join(format!("{image_name}.png"));
    let data =
        fs::read(&path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let b64 = base64_encode(&data);
    Ok(format!("data:image/png;base64,{b64}"))
}

/// Simple base64 encoder (avoids pulling in another crate).
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_versions,
            get_current_version,
            get_versions_dir,
            get_version_image,
        ])
        .setup(|app| {
            // --- Ensure bongo-versions dir exists & scan ---
            let vdir = versions_dir(&app.handle());
            let versions = scan_versions(&vdir);
            let config = load_config(&app.handle());
            let current = resolve_current_version(&config, &versions);

            println!(
                "[bongo-nat] Versions dir: {}",
                vdir.display()
            );
            println!(
                "[bongo-nat] Found {} custom version(s), selected: {}",
                versions.len(),
                current.name
            );

            // --- Build version CheckMenuItems ---
            let default_checked =
                current.name == DEFAULT_VERSION_NAME || current.path.is_none();
            let default_item = CheckMenuItem::with_id(
                app,
                "version:__default__",
                DEFAULT_VERSION_NAME,
                true,
                default_checked,
                None::<&str>,
            )?;

            let mut version_checks: Vec<CheckMenuItem<tauri::Wry>> =
                vec![default_item.clone()];

            for v in &versions {
                let checked = v.name == current.name && current.path.is_some();
                let item = CheckMenuItem::with_id(
                    app,
                    format!("version:{}", v.name),
                    &v.name,
                    true,
                    checked,
                    None::<&str>,
                )?;
                version_checks.push(item.clone());
            }

            // Build submenu from references
            let sub_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
                version_checks.iter().map(|c| c as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect();
            let version_submenu = Submenu::with_id_and_items(
                app,
                "versions_submenu",
                "Bongo Version",
                true,
                &sub_refs,
            )?;

            // --- Other tray items ---
            let open_versions = MenuItem::with_id(
                app,
                "open_versions",
                "Open Versions Folder",
                true,
                None::<&str>,
            )?;
            let show_bg = CheckMenuItem::with_id(
                app,
                "show_bg",
                "Show Background",
                true,
                false,
                None::<&str>,
            )?;
            let github =
                MenuItem::with_id(app, "github", "Visit GitHub page", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu =
                Menu::with_items(app, &[&version_submenu, &open_versions, &show_bg, &github, &quit])?;

            // Clone the version check items for the closure (cheap Arc clone).
            let version_checks_for_closure = version_checks.clone();

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Bongo Nat")
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| {
                    let id = event.id().as_ref().to_string();

                    if id == "quit" {
                        app.exit(0);
                    } else if id == "show_bg" {
                        let _ = app.emit("toggle-background", ());
                    } else if id == "open_versions" {
                        let dir = versions_dir(app);
                        let _ = tauri_plugin_opener::open_path(
                            &dir,
                            None::<&str>,
                        );
                    } else if id == "github" {
                        let _ = tauri_plugin_opener::open_url(
                            "https://github.com/pontusasp/bongo-nat",
                            None::<&str>,
                        );
                    } else if id.starts_with("version:") {
                        // --- Handle version selection ---
                        // 1. Toggle checks: only the selected item should be checked
                        for check in &version_checks_for_closure {
                            let is_selected = check.id().as_ref() == id;
                            let _ = check.set_checked(is_selected);
                        }

                        // 2. Determine which BongoVersion was selected
                        let is_default = id == "version:__default__";
                        let dir = versions_dir(app);
                        let all_versions = scan_versions(&dir);

                        let selected = if is_default {
                            BongoVersion {
                                name: DEFAULT_VERSION_NAME.to_string(),
                                path: None,
                            }
                        } else {
                            let version_name = &id["version:".len()..];
                            all_versions
                                .iter()
                                .find(|v| v.name == version_name)
                                .cloned()
                                .unwrap_or(BongoVersion {
                                    name: DEFAULT_VERSION_NAME.to_string(),
                                    path: None,
                                })
                        };

                        // 3. Save config
                        let new_config = Config {
                            selected_version: if selected.path.is_some() {
                                Some(selected.name.clone())
                            } else {
                                None
                            },
                        };
                        save_config(app, &new_config);

                        // 4. Notify frontend
                        let _ = app.emit("version-changed", &selected);
                        println!("[bongo-nat] Version changed to: {}", selected.name);
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
                                let _ = app_handle.emit("keypress", ());
                            }
                        }
                        rdev::EventType::KeyRelease(key) => {
                            held_keys.lock().unwrap().remove(&key);
                        }
                        rdev::EventType::ButtonPress(button) => {
                            let mut buttons = held_buttons.lock().unwrap();
                            if buttons.insert(button) {
                                let _ = app_handle.emit("keypress", ());
                            }
                        }
                        rdev::EventType::ButtonRelease(button) => {
                            held_buttons.lock().unwrap().remove(&button);
                        }
                        _ => {}
                    }
                }) {
                    eprintln!(
                        "[bongo-nat] ERROR: Could not start global keyboard listener: {:?}",
                        e
                    );
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
