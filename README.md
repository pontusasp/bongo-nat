# Bongo Nat

A desktop pet that bonks along to your keyboard and mouse input — a mashup of [Bongo Cat](https://store.steampowered.com/app/3419430/Bongo_Cat/) on Steam and **NAT**, the beloved mascot of the student society [ITK](https://itk.gg/). NAT exists in many variations, where often the origin of these variations is a pun (like Bongo Nat)!

Bongo Nat sits on top of all your windows as a small, transparent overlay. Every time you press a key or click a mouse button (anywhere on the system), nat bonks its paws. A lifetime bonk counter keeps track of how hard you've been working.

Built with [Tauri](https://tauri.app/), [SvelteKit](https://kit.svelte.dev/) and Rust.

## Features

- **Global input detection** — reacts to every keypress and mouse click system-wide, not just inside the app window.
- **Always-on-top transparent overlay** — a borderless, draggable nat that floats above everything else.
- **Bonk counter** — tracks your total bonks across sessions.
- **Custom bongo versions** — drop your own sprite sets into the `bongo-versions` folder and switch between them from the tray menu. Each version is a folder containing `idle.png`, `left.png` and `right.png`.
- **System tray integration** — all options are accessible by right-clicking the tray icon:
  - **Bongo Version** — switch between the built-in default and any custom versions you've added.
  - **Open Versions Folder** — opens the directory where you can add new bongo versions.
  - **Show Background** — toggles a circular background behind nat.
  - **Visit GitHub page** — opens the project repository in your browser.
  - **Quit** — exits the application.
- **Cross-platform** — runs on Linux, Windows and macOS.

## Adding Custom Versions

1. Right-click the tray icon and select **Open Versions Folder**.
2. Create a new subfolder (the folder name will appear in the menu).
3. Place three PNG images inside it: `idle.png`, `left.png`, `right.png`.
4. Restart Bongo Nat.
5. Right-click the tray icon → **Bongo Version** → select your new version.

## Known Issues

- **Sprites appear stacked on top of each other** — this can happen when the transparent window doesn't render correctly on some systems. To fix it, right-click the tray icon and enable **Show Background**. This adds a visible background behind nat which resolves the rendering issue.
- **Keypresses or mouse clicks are not always detected** — global input listening relies on low-level system hooks that may not work on all platforms or desktop environments.
