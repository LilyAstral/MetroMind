# MetroMind

A desktop app for brainstorming, for Windows and macOS. Ideas are **stations** on a **metro map**. Themes are colored **lines**, and an idea on two lines is an **interchange**. A **fisheye lens** follows the cursor, so the whole map always fits on screen. Each station glows with **energy** that rises when you work on it and fades gently over time, without any shame signals.

Built in Rust with [egui](https://github.com/emilk/egui) / eframe. The full product spec is in [instructions.md](instructions.md).

## Run it

```powershell
cargo run --release -p metromind-app
```

You can pass a map file to open it directly: `metromind-app path\to\ideas.metromind`. Without a path, the last map reopens. On first launch you get `My ideas.metromind` in the app data folder, so your first idea is saved without any dialog.

## Install it

```powershell
.\scripts\install.ps1
```

This builds a release binary and installs it for the current user (no admin rights needed): `MetroMind.exe` goes to `%LOCALAPPDATA%\Programs\MetroMind`, a Start Menu shortcut is added, and `.metromind` files open in MetroMind when double-clicked. Run it again to update; a running copy is closed first. Options: `-RunTests` (test before installing), `-DesktopShortcut`, `-NoFileAssociation`, `-InstallDir <path>`, and `-Uninstall` (keeps your maps and settings). If scripts are blocked, run `powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1`.

## Mac

No Apple developer account is needed. On a Mac with Rust and the Xcode Command Line Tools (`xcode-select --install`):

```sh
scripts/bundle-macos.sh
```

This builds a universal (Apple Silicon + Intel) `dist/macos/MetroMind.app` and `MetroMind-macos.zip`. Drag the app into Applications to install it. Without a Mac, use the **Release** workflow (see below).

The app is ad-hoc signed but not notarized, so a downloaded copy is blocked on first launch. Allow it under System Settings → Privacy & Security → **Open Anyway**, or run `xattr -dr com.apple.quarantine /Applications/MetroMind.app`. Copies you build yourself aren't blocked. Double-clicking a `.metromind` file doesn't open it in MetroMind on a Mac yet; use **File → Open**. Maps and settings live in `~/Library/Application Support/MetroMind`.

## Release builds

```powershell
.\scripts\bundle-windows.ps1
```

This writes `dist/windows/MetroMind-windows.zip`: a portable `MetroMind.exe` that runs from anywhere (maps and settings go to `%APPDATA%\MetroMind`). It isn't code-signed, so SmartScreen may warn on first launch: **More info → Run anyway**.

The **Release** GitHub workflow builds both the Windows zip and the Mac zip. Run it by hand to get them as artifacts (`gh workflow run release.yml`), or push a version tag to also publish a GitHub release with both attached:

```powershell
git tag v0.2.0
git push origin v0.2.0
```

## Checks

The checks below run at every milestone and all pass:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Timing benchmark (plan build for 500 stations):

```powershell
cargo test --release -p metromind-app -- --ignored --nocapture
```

## Using it

| Action | Input |
|---|---|
| New idea | Double-click empty space, then type |
| New idea on a line | Double-click a line (inserts between its two stations) |
| Rename | `F2`, double-click the station, or just start typing |
| Notes | `Enter` (opens the right-hand pane) |
| List items (in the notes pane) | `Enter` adds · `Backspace` on an empty item removes · `Alt+↑` / `Alt+↓` moves |
| Idea panel | Rest the pointer on an idea (it appears beside the cursor) |
| Move | Drag (snaps to the 40-unit grid on release); drag a selected idea to move the whole selection |
| Connect to an idea · branch off one | Right-drag from an idea onto another idea (connect) or onto empty space (branch), then pick a line |
| Context menu | Right-click an idea, a segment, or empty space |
| Select several | `Shift` + drag a box · `Shift` + click |
| Add to line / new line / remove | `1`–`9` / `N` / `Shift` + number (these act on the whole selection, except `N`) |
| Delete · rekindle | `Del` · `R` (both act on the whole selection) |
| Select all · find | `Ctrl+A` · `Ctrl+F` (then `Enter` to jump) |
| Clear selection or highlight | `Esc` |
| Pan · zoom | Drag empty space (or middle/right-drag) · wheel |
| Lens on/off · pin | `L` · hold `Space` |
| Lens strength · radius | `Ctrl` + wheel · `Shift` + wheel |
| Undo · redo | `Ctrl+Z` · `Ctrl+Y` (or `Ctrl+Shift+Z`) |
| New · open · save · save as | `Ctrl+N` · `Ctrl+O` · `Ctrl+S` · `Ctrl+Shift+S` |
| Fit whole map · shortcuts | `Home` · `F1` |

Every action is also in the menu bar and the right-click menus. **File → Load sample map** shows a ready-made brainstorm with 3 lines and 20 stations.

