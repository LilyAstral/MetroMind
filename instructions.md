# MetroMind: build instructions

> Working name. A Windows desktop app for brainstorming. Ideas are **stations** on a **metro map**, viewed through a **fisheye lens**, and each station gives off an **energy field** that shows where your attention has recently been.

---

## 0. Working agreement

**Mode:** `{STRICT}` ← the PM sets this before work starts. If it's unset, treat it as STRICT.

| Mode | Meaning |
|---|---|
| **STRICT** | Make no decisions of consequence on your own. At each milestone breakpoint, outline the steps you suggest and wait for approval. Build in small segments and test often. Raise any significant design decision. Make sure every piece works with the others before moving on. |
| **LOOSE** | Use your best judgment. Still follow the code standards in §8. |

**Prime directive.** These apply to every request, in this doc and in later conversation.
1. **Follow both the letter and the spirit of the intent.** Do what was asked, and do it in a way that serves *why* it was asked. If the literal wording and the obvious goal pull in different directions, don't quietly pick one. Point out the gap and propose how to resolve it.
2. **Suggest obvious improvements.** If a request has a clear improvement (a simpler approach, a missed edge case, a better default, a cheaper way to get the same result), say so briefly alongside the work. In STRICT mode, suggest it and wait for approval. In LOOSE mode, you may apply it if it's low-risk, but always say what you changed and why.

**Roles.** The human is the product manager. They own scope, UX, and priorities. You own implementation quality.

**Top priority: clean, manageable code.** When a fancier solution and a simpler one both meet the acceptance criteria, pick the simpler one. If a feature in this doc can only be built with tangled code, stop and raise it. Don't force it in.

---

## 1. Vision in one paragraph

Brainstorms are usually a mess of bubbles. MetroMind organizes them like a transit map. Each **theme** is a colored **line**. Each **idea** is a **station** on one or more lines, and a station on two lines is an **interchange**, which is an idea that connects two themes. The whole map always fits on screen. A **fisheye lens** follows the cursor and magnifies the area you're looking at while compressing everything else, so you never lose your place to zooming or panning. Stations glow with **energy** that rises when you work on them and fades gently over time. The map stays honest about where your attention goes and never punishes neglect.

Design for ADHD brains: low friction, nothing hidden permanently, no shame signals, and constant orientation.

---

## 2. Glossary

| Term | Definition |
|---|---|
| **Station** | One idea. It has a title, optional notes, a position, and energy. |
| **Line** | A named, colored theme. It's an ordered list of station IDs. |
| **Interchange** | A station that belongs to 2 or more lines. It isn't stored separately; it's derived. |
| **Segment** | The connection between two consecutive stations on a line. |
| **Lens** | The fisheye distortion centered on a focus point in screen space. |
| **Energy** | A 0–1 value per station that rises on interaction and decays exponentially. |
| **Layer** | A render pass that can be toggled independently (see §6). |

| Design Language | 
|---|---|
| **Clean** | Ikea. Sterile. Organized.
| **Pastel** | Light pastel blues, browns, and oranges.
| **Dark Mode** | Background should be dark, with a subtle texture. Text should be light. 
| **Best Practices** | Text contrast should be greater than 3:1 to surrounding background. Minimum clickable area must be greater than 44x44px for clickable elements.
| **Round** | Smooth, gradual corners and bends. 


---

## 3. Tech stack

- **Rust**, latest stable toolchain, **edition 2024**. Pin the channel in `rust-toolchain.toml` (`channel = "stable"`) and commit `Cargo.lock`.
- **egui** through **eframe** for the window, the immediate-mode UI, and all map drawing (through `egui::Painter`). Use eframe's default renderer.
- Target **Windows 10/11 x64** (`x86_64-pc-windows-msvc`). Nothing should be Windows-only unless it has to be. The code may well run elsewhere, but that isn't a goal.
- Add dependencies with `cargo add` so you get current versions. Don't hand-type version numbers from memory.

### Crates

| Crate | Where | Why |
|---|---|---|
| `eframe` (feature `persistence`) | app | Window, event loop, egui integration, per-user settings storage. Use egui through `eframe::egui` and **don't** add `egui` separately, so the versions can't drift apart. |
| `rfd` | app | Native Windows open/save dialogs. |
| `serde` (feature `derive`) | core, app | Serialization for the map file and app settings. |
| `serde_json` | core | The `.metromind` file format. |
| `chrono` (feature `serde`) | core | UTC timestamps for energy decay and created dates. |
| `glam` (feature `serde`) | core | `Vec2` math in world and screen space. Use `f32` to match egui. |
| `thiserror` | core | Typed errors (`MapError`). |
| `approx` | core (dev) | Float comparisons in tests. |

No other crates without PM approval. In particular, no async runtime, no ECS, no DI framework, and no extra GUI or widget crates.

---

## 4. Workspace structure

```
metromind/
├─ Cargo.toml                    # [workspace] members = ["crates/*"]
├─ rust-toolchain.toml
├─ crates/
│  ├─ metromind-core/            # lib. NO egui/eframe dependency.
│  │  ├─ src/
│  │  │  ├─ lib.rs
│  │  │  ├─ model/               # station.rs, line.rs, map.rs, ids.rs
│  │  │  ├─ geometry/            # camera.rs, lens.rs, router.rs, subdivide.rs
│  │  │  ├─ energy/              # model.rs, settings.rs
│  │  │  ├─ editing/             # edit.rs (enum Edit), undo.rs
│  │  │  ├─ persistence.rs       # load/save JSON, schema version
│  │  │  └─ clock.rs             # trait Clock, SystemClock
│  │  └─ tests/                  # integration tests (save/load round trip, etc.)
│  └─ metromind-app/             # bin. eframe app.
│     └─ src/
│        ├─ main.rs              # eframe::run_native, window options
│        ├─ app.rs               # struct MetroMindApp: eframe::App
│        ├─ settings.rs          # AppSettings (camera, lens, layers, last file)
│        ├─ input.rs             # egui input → Vec<Edit> + view changes
│        ├─ frame_plan.rs        # map + camera + lens + clock → FramePlan
│        ├─ render/              # one file per layer + colors.rs + glow.rs
│        └─ ui/                  # layers_panel.rs, notes_panel.rs, rename.rs, status.rs
```

**Rule:** all math, model logic, energy, undo, and persistence live in `metromind-core`, with unit tests. `metromind-app` is a thin shell that reads state, draws it, and turns input into edits.

---

## 5. Feature specification

### 5.1 Metro map (data and layout)

**Model** (in core, plain data, `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`):

```rust
pub struct StationId(pub u64);   // #[serde(transparent)], Copy, Ord, Hash
pub struct LineId(pub u64);

pub struct Station {
    pub id: StationId,
    pub title: String,
    pub notes: String,
    pub position: Vec2,              // world space, snapped to grid
    pub energy_at_touch: f64,        // 0..=1
    pub last_touched: DateTime<Utc>,
    pub created: DateTime<Utc>,
}

pub struct Line {
    pub id: LineId,
    pub name: String,
    pub color_index: u8,
    pub stations: Vec<StationId>,    // ordered
}

pub struct MetroMap {
    pub schema_version: u32,
    pub next_id: u64,                       // simple ID allocator
    pub stations: BTreeMap<StationId, Station>,
    pub lines: BTreeMap<LineId, Line>,
}
```

- `BTreeMap` gives a deterministic order, so saved files diff cleanly and tests are stable.
- Whether a station is an interchange is derived: `map.lines_through(id).count() >= 2`.

**Layout: octilinear routing.** Segments use only angles of 0°, 45°, and 90°, the classic metro look.
- Station positions snap to a world grid. The default cell is 40 units.
- A segment from A to B is routed as **one diagonal run plus one straight run**: go diagonally for `min(|dx|, |dy|)`, then straight for the rest. If `dx == 0` or `dy == 0`, it's a single straight run.
- The router is a **pure function**: `pub fn route(a: Vec2, b: Vec2) -> Vec<Vec2>` returns the polyline points (2 or 3 of them). No state, and no cache until profiling shows one is needed.
- When several lines share a segment, draw them as parallel offset strokes (offset = line index × stroke width). For the MVP it's fine to handle only the simple case where shared segments run between the same two stations.

**Out of scope for now:** automatic graph layout. Stations go where the user drops them, snapped to the grid.

### 5.2 Fisheye lens

The lens is a **render-time transform only**. It never changes model positions.

**Coordinate pipeline (every frame):**
```
world ──Camera──▶ view (px) ──FisheyeLens──▶ screen (px)
```
- `Camera { pan: Vec2, zoom: f32 }` handles pan and zoom (world → view pixels).
- `FisheyeLens` distorts view pixels around a focus point. Because it works in pixels, the lens feels the same at every zoom level.

**Math (Sarkar & Brown graphical fisheye), radial form:**

For a point p, with focus f, lens radius R, and distortion d ≥ 0:
```
r  = |p − f|
if r >= R or r == 0: p' = p                       (unchanged outside the lens)
x  = r / R                                         (normalized 0..1)
g  = (d + 1) * x / (d * x + 1)                     (distorted normalized radius)
p' = f + (p − f) * (g * R / r)
```
Local magnification, used to scale labels and hit radii:
```
m(x) = (d + 1) / (d * x + 1)^2      for x < 1, else 1
```
- Defaults: `R = 260 px`, `d = 3` (4× at the center). Both are adjustable.
- Implement it as `#[derive(Clone, Copy)] pub struct FisheyeLens { focus: Vec2, radius: f32, strength: f32 }` with `fn apply(&self, p: Vec2) -> Vec2` and `fn magnification(&self, p: Vec2) -> f32`. Unit-test both: identity outside R, monotonic inside, continuous at the boundary, and `m = d + 1` at the center.

**Curved lines.** A straight segment must bend through the lens. Before applying the lens, subdivide each routed polyline in **view space** into steps of ≤ 6 px, then apply the lens to every point. Put this in one pure helper: `fn subdivide(points: &[Vec2], max_step: f32) -> Vec<Vec2>`.

**Behavior**
- The focus follows the cursor with exponential smoothing so it doesn't jitter (`focus += (cursor − focus) * 0.25` per frame). While the focus hasn't caught up with the cursor (distance > 0.5 px), call `ctx.request_repaint()`. Once it has, stop repainting.
- `L` toggles the lens on and off. `Space` (hold) **pins** the lens where it is so you can move the mouse to a panel without the lens following.
- `Ctrl + mouse wheel` changes `d` (clamped to 0–6). `Shift + mouse wheel` changes `R` (clamped to 120–500 px). A plain wheel zooms the camera around the cursor.
- **Implementation note:** read the wheel from `egui::Event::MouseWheel { delta, modifiers, .. }` in `ctx.input(|i| i.events)` and use the larger of the two delta axes. Don't rely on `smooth_scroll_delta` or `zoom_delta`, because egui rewrites Ctrl+wheel and Shift+wheel for its own purposes.
- A thin lens ring outline is drawn as its own layer and can be toggled.

**Labels (level of detail)**
Show a station's label if **any** of these is true:
1. `zoom × m(p) ≥ LABEL_THRESHOLD` (default 1.6)
2. The station is selected or hovered
3. The station's current energy is ≥ 0.6 (hot ideas are always readable)

Label font size = `BASE_FONT_PX × clamp(m(p), 1.0, 2.2)`, drawn with `painter.text(..., FontId::proportional(size), ...)`.

**Hit testing** happens in **screen space** against the distorted positions in the current `FramePlan`. The hit radius is `BASE_HIT_PX × m(p)`. There's no inverse transform, which keeps it simple.

### 5.3 Energy field

**Model: lazy exponential decay.** Nothing ticks, and no background timers run.
- Each station stores `energy_at_touch` (0–1) and `last_touched`.
- Current energy is computed when needed:
  ```
  E(now) = energy_at_touch * 0.5 ^ ((now − last_touched) / half_life)
  ```
- A **bump** on interaction does this: `E = min(1, E(now) + amount)`, then stores `energy_at_touch = E` and `last_touched = now`.
- All time comes from `trait Clock { fn now(&self) -> DateTime<Utc>; }`. The app uses `SystemClock`, and tests use a `FakeClock` they can move forward by hand.
- Energy math uses `f64`, converting durations with `num_milliseconds() as f64 / 1000.0`.

**Default settings** (one `EnergySettings` struct with a `Default` impl, user-tunable later):

| Setting | Default |
|---|---|
| `half_life` | 3 days |
| New station starts at | 0.6 |
| Select / open | +0.05 |
| Edit title or notes | +0.20 |
| Add to a line / connect | +0.15 |
| Drag to a new position | +0.05 |
| Visual floor (rendering only) | 0.08 |

The visual floor means a cold station is still clearly drawn. Cold is quiet, not gone.

**Visuals.** This is a separate layer, drawn *beneath* the stations:
- A **halo** around each station: radius = `STATION_RADIUS × (1.5 + 3 × E)` and center alpha = `0.15 + 0.55 × E`, scaled by the lens magnification. egui has no radial gradients, so build each halo as an `egui::Mesh` **triangle fan** (about 24 rim vertices): the center vertex gets the full color and the rim vertices get alpha 0, so vertex interpolation draws the gradient. Keep this in one helper, `render/glow.rs::radial_glow(center, radius, color) -> Mesh`.
- A **color temperature ramp**, cool → warm: slate blue (cold) → teal → amber (hot), as `render/colors.rs::energy_color(e: f32) -> Color32`. **Never red.** Red reads as an error or "overdue," which is exactly the shame signal to avoid.
- Optional **field layer** (off by default): a soft heatmap. Compute it on the CPU into a low-resolution grid (about 1/8 of screen resolution) by splatting each station's falloff over only the cells within its radius. Upload it as an `egui::ColorImage` to a cached `TextureHandle` and draw it stretched over the map. Recompute it **only when the plan changes** (map edited, camera moved, lens moved), using a simple dirty flag.
- Decay is slow, so the app doesn't need to animate it. Call `ctx.request_repaint_after(Duration::from_secs(60))` so colors refresh occasionally while the app sits idle.

**No-shame rules**
- Nothing shows "days since touched," counts, or warnings.
- Decay never removes, hides, or reorders anything.
- An optional "rekindle" action (`R` on a selected station) bumps it +0.3. This turns revisiting old ideas into a small positive act.

---

## 6. Layers

egui redraws everything each frame, so a layer is just a **plain function**, not a trait object:

```rust
fn draw_grid(painter: &Painter, plan: &FramePlan, theme: &Theme)
```

Draw them in this fixed order from `render/mod.rs`, skipping any layer whose flag is off in `LayerVisibility` (a struct of `bool`s that derives `Serialize`/`Deserialize` and is saved in `AppSettings`):

1. **Grid**: faint dots at grid intersections, distorted by the lens. Default: on.
2. **Energy field**: the heatmap texture. Default: off.
3. **Halos**: per-station energy glow meshes. Default: on.
4. **Lines**: subdivided, distorted, octilinear strokes (`Shape::line`). Default: on.
5. **Stations**: circles for regular stations, a white pill with a dark outline for interchanges. Default: on.
6. **Labels**: following the LOD rules in §5.2. Default: on.
7. **Lens ring**: the outline of the lens radius. Default: on.

`FramePlan` is an **owned** struct built once per frame. It holds the camera, lens, a `Vec<ScreenStation { id, pos, magnification, energy, is_interchange }>`, and a `Vec<ScreenLine { color_index, points }>`. Layers **only read** from it. Because it's owned, `FramePlan` needs no lifetimes.

A **Layers panel** (`egui::SidePanel::left`, collapsible) shows one checkbox per layer.

---

## 7. Architecture rules

**Per-frame flow** inside `MetroMindApp::update` (immediate mode, top to bottom, no hidden callbacks):

```
1. input::collect(ctx, &plan_from_last_frame) -> InputOutcome { edits: Vec<Edit>, view: ViewChange }
2. for edit in edits: undo_stack.apply(&mut map, edit, &clock, &energy_settings)
3. apply ViewChange to camera / lens / selection
4. let plan = frame_plan::build(&map, camera, lens, &clock, &energy_settings)
5. side panels (layers, notes) → may produce more edits, queued for next frame
6. CentralPanel: render::draw_all(&painter, &plan, &layers, &theme)
7. autosave check; repaint scheduling
```

- **One-way flow:** Input → `Edit` → Model → next frame's plan → Render. Render code never touches `MetroMap`.
- **Edits are an enum, not trait objects.** `enum Edit { AddStation{..}, MoveStation{..}, Rename{..}, AddToLine{..}, ... }`. `fn apply(self, map: &mut MetroMap) -> Edit` makes the change and **returns its inverse**. `UndoStack` keeps two `Vec<Edit>`s (undo and redo). The `match` is exhaustive, so adding an edit forces you to handle it everywhere. This is also the single place where energy bumps happen.
- **Keyboard shortcuts must not fire while typing.** Skip shortcut handling whenever `ctx.wants_keyboard_input()` is true, for example while the rename box or notes pane has focus.
- **Repaint only when needed:** egui already repaints on input. Request extra repaints only for lens smoothing (§5.2), pending autosave, and the 60 s idle refresh.
- **No global state:** no `static mut`, no lazy globals, no `Rc<RefCell<…>>` webs. `MetroMindApp` owns the map, undo stack, camera, lens, settings, and clock, and passes `&`/`&mut` down.

---

## 8. Code standards

- `#![forbid(unsafe_code)]` in both crates.
- `cargo fmt` clean, and `cargo clippy --all-targets -- -D warnings` clean, at every breakpoint.
- **No `unwrap()` or `expect()`** outside tests and `main.rs` startup. Core returns `Result<_, MapError>`. The app shows errors in a status line and never panics.
- Modules stay focused: aim for under ~200 lines per file. Functions stay short: aim for under ~30 lines. Pull math into named, pure functions.
- **Pure functions for all geometry and energy math.** No side effects, and no hidden time (always take `now` or `&dyn Clock`).
- Small value types (`Vec2`, `Camera`, `FisheyeLens`, IDs, `ScreenStation`) derive `Copy`. Borrow before you clone.
- Use enums for closed sets (`Edit`, layer kinds) and traits only for real seams (`Clock`). Avoid generics and lifetimes in public types unless they actually remove complexity.
- Name things after the domain (`Station`, `Interchange`, `rekindle`), not generic words (`Node`, `Item`, `Manager`).
- `///` doc comments explain **why**. Put the formulas from this doc in the doc comments of the functions that implement them.
- Keep all tunable numbers in a few named settings structs or `const`s at the top of the relevant module, never inline.
- Every core feature ships with unit tests (`#[cfg(test)] mod tests` beside the code, plus integration tests in `crates/metromind-core/tests/`). App code is verified by running it and following the milestone checklist.
- Hide the console in release builds with `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` in `main.rs`.

---

## 9. Persistence

- Save format: a single `.metromind` file that is UTF-8, pretty-printed JSON (`serde_json::to_string_pretty`).
- Autosave 2 seconds after the last edit (store an `Option<Instant>`, check it in `update`, and use `request_repaint_after` so the check actually runs). `Ctrl+S` saves immediately.
- Write to `<name>.metromind.tmp` in the same folder, then `std::fs::rename` over the original. That swap is atomic, so a crash can never corrupt the map. No extra crate is needed.
- Include `schema_version`, and refuse to load newer versions with a friendly message.
- Open and save-as go through `rfd::FileDialog`. Run the dialogs synchronously, which is fine for this app.

```json
{
  "schema_version": 1,
  "next_id": 6,
  "stations": {
    "1": { "id": 1, "title": "Offline mode", "notes": "",
           "position": [120.0, 80.0],
           "energy_at_touch": 0.72,
           "last_touched": "2026-11-23T10:40:00Z",
           "created": "2026-11-20T08:00:00Z" }
  },
  "lines": {
    "2": { "id": 2, "name": "Sync", "color_index": 0, "stations": [1, 4] }
  }
}
```

Camera, lens settings, layer visibility, and the last opened file live in `AppSettings`. Save and load it with eframe's persistence (`eframe::set_value` / `get_value` under `eframe::APP_KEY`), not in the map file.

---

## 10. Interactions (MVP)

| Action | Input |
|---|---|
| New station at cursor | Double-click empty space |
| New station on a line | Double-click a segment (inserts it between those two stations) |
| Rename | `F2` or start typing with a station selected (an inline `TextEdit` in an `egui::Area` at the station) |
| Notes | `Enter` on a selected station opens the notes pane (`SidePanel::right`) |
| Move station | Drag (snaps to the grid on release; one `MoveStation` edit per drag, not per frame) |
| Add selected station to a line | `1`–`9` = line by index; `N` = new line starting here |
| Remove from line | `Shift` + the line's number |
| Delete station | `Del` (undoable) |
| Rekindle | `R` |
| Pan | Middle-drag, or right-drag on empty space |
| Zoom | Mouse wheel |
| Lens on/off · pin | `L` · hold `Space` |
| Lens strength · radius | `Ctrl` + wheel · `Shift` + wheel |
| Undo · redo | `Ctrl+Z` · `Ctrl+Y` |
| Open · save · save as | `Ctrl+O` · `Ctrl+S` · `Ctrl+Shift+S` |
| Fit whole map | `Home` |

---

## 11. Milestones

Each milestone ends at a **breakpoint**. In STRICT mode, stop there, show what works, and propose the next steps. At every breakpoint, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` must all pass.

**M0: Scaffold**
- The workspace, both crates, `rust-toolchain.toml`, an eframe window with an empty `CentralPanel`, and one passing test in core.
- ✅ `cargo run -p metromind-app` opens a window. `cargo test` passes. Clippy is clean.

**M1: Core model and persistence**
- `Station`, `Line`, `MetroMap`, IDs, `persistence::{load, save}`, `Clock`, and `MapError`.
- ✅ Round-trip tests pass (save → load → equal). A sample map file loads. A newer schema version is rejected cleanly.

**M2: Static metro map**
- `Camera`, `route`, `frame_plan::build`, and the Grid, Lines, Stations, and Labels layers. No lens yet.
- ✅ A hard-coded sample map with 3 lines, about 20 stations, and 2 interchanges renders correctly. Router unit tests pass.

**M3: Editing and undo**
- `input.rs`, `enum Edit`, `UndoStack`, pan and zoom, rename and notes UI, autosave, and file dialogs.
- ✅ Every action in §10 except the lens rows works and can be undone. Shortcuts don't fire while you're typing.

**M4: Fisheye lens**
- `FisheyeLens`, `subdivide`, lens smoothing, pin, wheel handling with modifiers, LOD labels, magnified hit testing, and the Lens ring layer.
- ✅ The lens unit tests (§5.2) pass. Lines bend smoothly. Clicking a magnified station hits the right one. A release build with 500 stations holds 60 fps on a mid-range laptop. CPU use is near idle when the mouse is still.

**M5: Energy field**
- Energy model, bumps applied in `UndoStack::apply`, the halo mesh, the color ramp, Rekindle, and the optional Field layer with its cached texture.
- ✅ Decay and bump tests with `FakeClock` pass. Hot stations are visibly warmer and always labeled. No red appears anywhere.

**M6: Layers panel and polish**
- Layer checkboxes, "fit map," `AppSettings` persistence, and an empty-state hint ("Double-click anywhere to drop your first idea").
- ✅ The PM walkthrough passes.

---

## 12. ADHD-friendly UX checklist

Check every milestone against this list:
- [ ] Capturing an idea takes one gesture (double-click, then type). No dialogs.
- [ ] Nothing ever leaves the screen. The whole map is always reachable with `Home`.
- [ ] The selected station and the lens focus are always obvious, so "where am I" is answered at a glance.
- [ ] No red, no counters, no overdue language.
- [ ] Motion is smooth but short (≤ 200 ms). A "reduce motion" setting turns off lens smoothing and halo transitions.
- [ ] Every destructive action can be undone.

---

## 13. Later (do not build yet)

- Global quick-capture hotkey and tray icon (an inbox that feeds a "Holding" line)
- Time scrubber that replays how the map grew
- Automatic octilinear layout
- Pen and ink input
- Multiple maps and tabs
- Export to PNG and SVG
- App icon and installer (for example `embed-resource` plus an MSI or MSIX package)

---

## 14. Open questions for the PM

1. Should energy half-life be one global setting, or tunable per map?
2. Can a station exist on **no** line (a free-floating idea), or must it always belong to one? *(Suggested default: allowed, drawn as a hollow circle.)*
3. Is there a cap on the number of lines? *(Suggested: 9, to match the number-key shortcuts.)*
4. Light theme, dark theme, or follow the Windows setting? *(egui defaults to following the system theme.)*