# MetroMind

A Windows desktop app for brainstorming. Ideas are **stations** on a **metro map**. Themes are colored **lines**, and an idea on two lines is an **interchange**. A **fisheye lens** follows the cursor, so the whole map always fits on screen. Each station glows with **energy** that rises when you work on it and fades gently over time, without any shame signals.

Built in Rust with [egui](https://github.com/emilk/egui) / eframe. The full product spec is in [instructions.md](instructions.md).

## Run it

```powershell
cargo run --release -p metromind-app
```

You can pass a map file to open it directly: `metromind-app path\to\ideas.metromind`. Without a path, the last map reopens. On first launch you get `My ideas.metromind` in the app data folder, so your first idea is saved without any dialog.

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

## Layout

```
crates/
  metromind-core/   library, no GUI dependency, all logic and its tests
    model/          Station, Line, MetroMap, IDs
    geometry/       Camera, FisheyeLens, octilinear route, subdivide, polyline helpers
    energy/         lazy exponential decay, bumps, EnergySettings
    editing/        enum Edit (apply returns its inverse), UndoStack
    persistence.rs  .metromind JSON, schema version, atomic save
    clock.rs        Clock trait, SystemClock, FakeClock
    sample.rs       the sample map
    tests/          integration tests (save/load round trip, editing sessions)
  metromind-app/    thin eframe shell
    app/            MetroMindApp: per-frame flow, commands, file handling
    input/          egui input → Vec<Edit> + Vec<Command>
    frame_plan/     map + camera + lens + clock → FramePlan (screen space)
    render/         one plain function per layer, plus colors.rs and glow.rs
    ui/             menu, side panel, notes pane, rename box, status line, help
    document.rs     the open file: map, undo history, autosave timing
    settings.rs     AppSettings (eframe persistence)
```

Data flows one way: input → `Edit` → model → this frame's `FramePlan` → render. Render code never touches `MetroMap`. Saving writes `<name>.metromind.tmp` and then renames it over the original, so a crash can't corrupt a map. Autosave runs 2 seconds after the last change.

## M7: connect, select, and organize

- **Connect and branch.** Right-drag from an idea onto another idea to connect them, or onto empty space to branch off a new idea. A line picker opens: scroll with the wheel, click a line, or press `1`–`9`, `N` (new line), or `Enter`. Lines that can't take the connection are greyed out with the reason.
- **Remove connection.** Right-click a segment. Cutting the end of a line just shortens it; cutting the middle **splits the line in two**. Undo restores it.
- **Views.** The side panel's "Layers" section is now **Views**. Lines can be reordered with ▲ / ▼, and the number keys follow that order. Clicking a line's number highlights it on the map.
- **Palettes and colours.** Six palettes, shown as striped icons above the line list. Clicking a line's colour swatch picks a custom colour; ↺ returns it to the palette colour.
- **Map file schema v2.** Saves write `schema_version: 2`, which adds `line_order`, `palette`, and each line's `custom_color`. **Version 1 files still open.** On load their lines take ascending ID order, the Pastel palette, and no custom colours. Files from a newer version are refused with a message.

## Decisions on the spec's open questions (§14)

1. **Energy half-life** is one global setting, part of `EnergySettings` inside `AppSettings`. It's easy to move into the map file later.
2. **Free-floating ideas** are allowed and are drawn as a hollow ring. Capturing an idea shouldn't require choosing a theme first.
3. **Lines are capped at 9**, matching the number keys. `N` says so gently when all 9 are in use.
4. **Theme** is always dark, as the design language asks.

## Where the build differs from the spec, and why

- **egui 0.37 API.** `eframe::App::update` became `App::ui`, and `SidePanel`/`TopBottomPanel` became `egui::Panel`. The per-frame flow is otherwise the one in §7.
- **`Edit::apply` returns `Result<Edit, MapError>`** rather than a bare `Edit`. An edit that names a missing station shows a message in the status line instead of panicking.
- **Energy bumps.** Edits bump energy in `UndoStack::apply`. Selecting and rekindling bump through `editing::touch` and aren't undoable, because they record attention rather than change the map. Undo and redo never bump.
- **Start typing to rename.** Letters that are shortcuts (`L`, `N`, `R`, digits) run the shortcut. Any other character starts a rename with that character.
- **Additions:**
  - Left-drag on empty space pans.
  - Double-clicking a station renames it.
  - `Esc` deselects, `Ctrl+N` makes a new map, and `F1` shows a cheat sheet.
  - Bends are rounded, following the "Round" design rule.
  - Stations compressed near the lens rim keep at least 60% of their hit radius.
  - The sample map only loads into an empty map, so it can't overwrite ideas.
  - A file other than the last one opens fitted to the window.
- **Lens off** is represented as strength 0 (the identity), so there's one code path.
- **M7 deviations.** Palette icons are square stripes with square corners (the 4 px rounding isn't drawn). Each change in the colour picker is its own undo step, so one colour pick can take several undos.

## Performance

Measured in a release build on the development machine, with 500 stations on 9 lines and every layer on:

- Building the frame plan takes about 0.8 ms per frame.
- A whole frame takes about 8.5 ms of CPU.
- Idle CPU stays near zero: repaints happen only for input, lens smoothing, a pending autosave, and a once-a-minute refresh.
