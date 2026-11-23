# MetroMind M7: Connect, select, and organize

> Follow-on to `instructions.md`. Everything there still applies, especially §7 (architecture), §8 (code standards), and §12 (ADHD checklist). This file adds features only. Where it changes something in `instructions.md`, this file wins.

---

## 0. Working agreement

**Mode: LOOSE inside this spec.** The PM has approved everything here. Work through the milestones in order without waiting for approval at each breakpoint, but **stop and report** if:
- a feature can only be built with tangled code (raise it, don't force it in),
- you need a new crate (egui's built-in widgets, including the colour picker, are allowed), or
- a requirement here conflicts with `instructions.md` in a way §0 doesn't settle.

**Each breakpoint:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass. Then make a commit named `M7.x: <summary>`. Work on a branch, not `master`.

**Terminology:** in the UI, **line** is the word for a named, coloured theme. The PM sometimes calls these "layers". The render toggles (Grid, Halos, …) are renamed **views** in M7.5.

---

## 1. Summary of features

| # | Feature | Milestone |
|---|---|---|
| A | `Edit::Batch` so multi-step actions undo in one step | M7.1 |
| B | Multi-select: Shift+drag box, Shift+click toggle, group move, a ghost marker where stations will land | M7.2 |
| C | Right-drag from a station to connect it to another station, with a scrollable line picker | M7.3 |
| D | Branch off an idea: right-drag onto empty space to create a connected new station | M7.3 |
| E | Right-click context menus on stations, segments, and empty space, including **Remove connection** | M7.4 |
| F | Side panel: rename "Layers" to "Views"; reorder lines; colour palettes with striped icons; custom line colours; highlight a line | M7.5 |
| G | Search and jump (`Ctrl+F`) | M7.6 |
| H | Hover preview of a station's notes | M7.6 |

---

## 2. Core changes (M7.1)

All of this lives in `metromind-core`, with unit tests. Nothing here depends on egui.

### 2.1 `Edit::Batch(Vec<Edit>)`

- `apply` applies the edits in order and collects their inverses. If any edit fails, it applies the inverses collected so far in reverse order, then returns the error. On error, the map is unchanged.
- The inverse is `Edit::Batch(inverses_reversed)`.
- Replace `Edit::touch() -> Option<..>` with `Edit::touches() -> Vec<(StationId, Touch)>`. A batch returns the touches of its children, with one entry per station (first one wins). `UndoStack::apply` bumps each station.
- Input code never sends an empty batch. A batch with exactly one edit is fine.
- Tests: round trip, rollback on a failure partway through, and energy bumps (each station bumped once).

### 2.2 Line order

The number keys bind to lines in a user-chosen order, not ID order.
- Add `MetroMap.line_order: Vec<LineId>`. Invariant: it is a permutation of `lines`' keys. `validate()` checks this.
- `line_at(index)` reads `line_order`. Add `ordered_lines()`, and use it everywhere the app lists lines.
- `Edit::CreateLine { line, position: usize }` inserts into `line_order` at `position`. The inverse of `DeleteLine` restores the original position.
- New edit `Edit::MoveLine { id, to: usize }`. Its inverse is `MoveLine` back to the old index.
- Move `free_color` from `input/keys.rs` to `MetroMap::free_color_index()`.

### 2.3 Colours: palette and custom colours

- Add `MetroMap.palette: u8`, an index into the app's palette list. It defaults to 0. It's stored in the map file so a map keeps its look.
- Add `Line.custom_color: Option<[u8; 3]>`. When it's set, it overrides the palette colour for `color_index`.
- New edits:
  - `Edit::SetPalette { palette: u8 }`
  - `Edit::SetLineColor { id, custom: Option<[u8; 3]> }`. `None` resets the line to its palette colour.
- Both edits are undoable. Neither one bumps energy.

### 2.4 Schema v2

- Change `SCHEMA_VERSION` to 2.
- v1 files still load: `line_order` defaults to ascending IDs, `palette` to 0, and `custom_color` to `None`. Use `#[serde(default)]` plus a fix-up step after loading.
- Saves always write v2. Files newer than v2 are still refused with a friendly message.
- Tests: a v1 fixture loads correctly, a v2 save→load round trip is exact, and v3 is refused.

### 2.5 Connect rules (`editing/connect.rs`)

`pub fn connect(map, line: LineId, a: StationId, b: StationId) -> Result<Edit, ConnectRefusal>`. Here `a` is the station the drag started from.

| Situation on `line` | Edit |
|---|---|
| `a` is its last station | `AddToLine` of `b` at the end |
| `a` is its first station (and not the last) | `AddToLine` of `b` at index 0 |
| `a` is in the middle | `AddToLine` of `b` at `index_of(a) + 1`. The line now runs through `b`. |
| `a` isn't on the line but `b` is | The same rules with `a` and `b` swapped |
| Neither station is on the line | `Refusal::NeitherOnLine` |
| Both are on the line | `Refusal::BothOnLine`. This also covers the case where they're already adjacent. |

`ConnectRefusal` has a `reason(&self, line_name) -> String` method that returns calm, short text, for example "Neither idea is on Sync yet". The picker shows this text on greyed-out rows.

Also add `pub fn connect_new_line(map, a, b) -> Result<Edit, ConnectRefusal>`. It returns `CreateLine` with `[a, b]`, the next free colour, the name "Line N", and the position at the end. It returns `Refusal::NoFreeLine` when all 9 lines are in use.

### 2.6 Remove a connection (`editing/connect.rs`)

`pub fn disconnect(map, line, index) -> Result<Edit, ConnectRefusal>` removes the segment between `stations[index]` and `stations[index + 1]`. The line has `n` stations.

| Case | Edit |
|---|---|
| `n == 2` | Remove the second station from the line. The line keeps one station. |
| The first segment (`index == 0`) | Remove station 0 from the line |
| The last segment | Remove the last station from the line |
| A middle segment | **Split the line**. `Batch[RemoveFromLine × tail (from the end), CreateLine { tail, next free colour, "<name> 2", position right after this line }]` |
| A middle segment with 9 lines already | `Refusal::NoFreeLine`, with the text "All 9 lines are in use, so this line can't be split." |

Removing a connection never deletes a station.

### 2.7 Search (`search.rs`)

`pub fn search(map, query: &str, limit: usize) -> Vec<StationId>`. Matching is case-insensitive and ignores surrounding whitespace. Results are ranked:
1. title starts with the query,
2. title contains it,
3. notes contain it.

Within each rank, order by title, then by ID. An empty query returns nothing.

---

## 3. Multi-select (M7.2)

- Replace `Interaction.selected: Option<StationId>` with:
  - `selection: BTreeSet<StationId>`
  - `primary: Option<StationId>`: the station clicked most recently, which is always in the selection. It's drawn with the existing selected style; the other selected stations get a slightly dimmer ring.
- **Click a station:** select only that station. **Shift+click a station:** add or remove it from the selection. **Click empty space:** clear the selection. **`Esc`** also clears it, when no text box or popup is open. **`Ctrl+A`:** select all.
- **Shift+left-drag on empty space:** draw a selection box. It's a rounded, translucent rectangle in the accent colour. Stations whose *screen* position (after the lens) is inside the box are **added** to the selection on release. Plain left-drag on empty space still pans.
- **Group move:** left-dragging a selected station moves every selected station by the same world offset. On release, each station snaps to the grid on its own. All the moves go into one `Batch` of `MoveStation`. Dragging an unselected station selects only it and moves it, as before.
- **Ghost marker:** while dragging, draw a hollow, faint circle at each station's snapped landing spot, distorted by the lens. Add it to `FramePlan` as `ghosts: Vec<Vec2>` and render it in its own small function, drawn just under the stations.
- **Actions on the selection:**
  - `Del`, `R`, `1`–`9`, and `Shift+number` apply to every selected station, as one `Batch`. Stations where the action does nothing are skipped. Only if every station would be skipped, show a notice.
  - `F2`, `Enter`, typing to rename, and `N` act on `primary` only.

---

## 4. Connect by right-dragging (M7.3)

### 4.1 Gesture

- **Right-drag that starts on a station** starts a connection. **Right-drag that starts on empty space** still pans. A right-click without dragging opens the context menu (M7.4).
- **While dragging,** draw a **connector preview**. It's a dashed, octilinear line from the origin station to the cursor, routed with `route()` and bent by the lens like a real segment, in the accent colour. When the cursor is over another station, the preview snaps to that station, and the station gets the hover ring. Put the preview in `FramePlan` (`connector: Option<Vec<Vec2>>`) and draw it in `render/connector.rs`, above the lines and below the stations.
- **On release:**
  - Over **another station** → open the picker in connect mode.
  - Over **empty space** → open the picker in branch mode (§4.3). The new station's spot is the grid-snapped cursor position. If that cell already holds a station, use connect mode with that station instead.
  - Over **the origin station**, or after a drag shorter than egui's drag threshold → cancel, and nothing happens.
- The connector preview stays visible while the picker is open.

### 4.2 Line picker (`ui/line_picker.rs`)

A small `egui::Area` that opens at the release point. It's moved if needed so it stays fully inside the map viewport. It uses rounded corners and the dark theme.

- **Rows:** one per line in `line_order` (colour swatch, number, and name), then **"+ New line"**. Each row is at least 44 px tall.
- **Lines that can't take the connection** (a refusal from §2.5) are greyed out, with the reason in small text below the name. They stay visible.
- **Starting highlight:** if exactly one valid line contains the origin station, that line. Otherwise, the first valid row.
- **Mouse wheel:** moves the highlight one row per notch, skipping greyed rows. It stops at the ends (no wrapping). Hovering a row also highlights it.
- **Picking:** left-click a valid row, press `Enter` for the highlighted row, press `1`–`9` for a line by number, or press `N` for New line.
- **Cancelling:** `Esc`, a click outside the picker, or a right-click. Nothing changes.
- **While the picker is open,** the map ignores the wheel (no zoom or lens changes), panning, clicks, and shortcuts. Skip `pointer::wheel` and `keys::collect` when the picker is open.
- **A pick** produces the edit from §2.5, applies it, selects the target station, and closes the picker. The connected stations get the Connect energy bump.

### 4.3 Branch mode: branch off an idea

The same picker, with different results:
- **An existing line** → `AddStation { station, memberships: [Membership { line, index }] }`. The index comes from the §2.5 rules, with the new station as `b`, so it is valid whenever the origin station is on the line. Lines that don't contain the origin station are greyed out with the reason "This idea isn't on <name>".
- **"+ New line"** → `Batch[AddStation (no memberships), CreateLine [origin, new]]`.
- Then select the new station and open its rename box straight away, the same as double-clicking empty space. That keeps it to one gesture: drag, pick, type.

---

## 5. Context menus (M7.4)

Use egui's `Response::context_menu`. Menu items are at least 44 px tall. Each item that has a shortcut shows it on the right. Menus act on the thing under the cursor at the moment of the right-click.

**Station** (if the station isn't selected, right-clicking selects only it first):
- Rename (`F2`) · Notes (`Enter`)
- **Add to line ▸**: a submenu of lines, with lines the station is already on greyed out. **Remove from line ▸**: only the lines the station is on.
- Start a new line (`N`) · Rekindle (`R`)
- Delete (`Del`)
- When more than one station is selected, Add to line, Remove from line, Rekindle, and Delete act on the whole selection, and the menu title says "3 ideas selected".

**Segment** (use the existing `segment_at` hit test; the title is the line's name in its colour):
- **Remove connection**: uses §2.6. When it's refused, the item is greyed out and its hover text gives the reason.
- Insert idea here: does what double-clicking the segment does.
- Highlight this line (§6)

**Empty space:**
- New idea here: does what double-clicking does.
- Fit map (`Home`)
- Select all (`Ctrl+A`)

---

## 6. Side panel (M7.5)

### 6.1 Views

- Rename the "Layers" heading to **"Views"**. Rename the code to match: `Layer` → `View`, `LayerVisibility` → `ViewVisibility`, and `layers_panel.rs` → `side_panel.rs` (split it into files if it grows past about 200 lines).
- Keep existing saved settings readable by putting `#[serde(alias = "layers")]` on the renamed `AppSettings` field.
- Update the help overlay and the README to match.

### 6.2 Lines section

Each row, from left to right:
- **Colour swatch:** clicking it opens egui's built-in sRGB colour picker, which sets a custom colour. If a custom colour is set, a small "↺" button next to the swatch resets the line to its palette colour.
- **Number**
- **Name field**: unchanged.
- **▲ / ▼ buttons:** these send `MoveLine`. They're disabled at the ends of the list.
- **✕:** unchanged.

All row buttons meet the 44×44 px hit target, even if their icon is smaller.

**Highlight:** hovering a line row highlights that line on the map. Clicking the row's number toggles a lasting highlight; `Esc` or clicking it again clears it. While a line is highlighted, every other line and its stations are drawn at 35% alpha. This is view-only state in the app, not saved in the map or settings. Add `dimmed: bool` to `ScreenLine` and `ScreenStation`.

### 6.3 Palettes

- Show a **Palette** row above the lines list: one icon per palette, with its name as hover text. The selected palette gets an accent outline. Clicking an icon sends `SetPalette`.
- **Icon:** a 24 px square with rounded corners (4 px), filled with **45° diagonal stripes** that repeat the palette's 9 colours in order. Each stripe is about 3 px wide. Its hit area is 44 px. Draw it with one helper, `ui/swatch.rs::striped_swatch(painter, rect, &[Color32])`: clip to the rect and draw thick 45° line segments.
- Palette definitions live in `render/colors.rs` (or a new `render/palettes.rs`) as `const PALETTES: [Palette; 6]` with `name` and `colors: [Color32; 9]`. Replace `line_color(index)` with one resolver that the frame plan, panel, and picker all use: `line_color(map, line) -> Color32`, where a custom colour beats the palette colour.

| Palette | Colours (sRGB) |
|---|---|
| **Pastel** (default; today's colours) | (142,197,232) (244,177,131) (200,162,124) (143,209,193) (185,169,230) (235,213,142) (168,221,160) (169,184,216) (227,201,168) |
| **Harbour** | (126,180,226) (160,206,236) (98,170,196) (140,214,214) (186,196,232) (118,150,206) (204,222,240) (214,196,160) (170,190,170) |
| **Terracotta** | (232,160,112) (246,196,150) (196,146,104) (224,182,128) (250,214,170) (214,168,140) (186,160,128) (236,200,120) (206,184,160) |
| **Meadow** | (160,210,150) (196,226,160) (130,196,170) (214,220,140) (150,200,200) (186,210,186) (226,206,150) (170,186,140) (140,180,150) |
| **Dusk** | (190,170,232) (214,186,230) (160,170,226) (200,200,240) (176,196,230) (226,206,236) (150,180,210) (210,190,210) (186,176,206) |
| **High contrast** | (102,194,255) (255,184,77) (120,220,140) (200,160,255) (255,230,110) (80,220,210) (230,230,230) (255,150,200) (180,200,120) |

**Palette tests:**
- Every palette colour has a contrast ratio of at least 3:1 against `Theme::background`.
- No palette colour is "red": a hue within 15° of 0° and HSV saturation above 0.35 fails.

If a colour in the table fails either test, adjust it slightly and say so in the commit message.

Custom colours are the user's explicit choice and aren't restricted.

---

## 7. Search and notes preview (M7.6)

### 7.1 Search (`ui/search.rs`)

- `Ctrl+F` opens a search box at the top centre of the map, as an `egui::Area` with keyboard focus. Because it takes focus, shortcuts are suppressed while you type, through the existing `typing` flag.
- Show up to 8 results from `core::search` below the box, each as a 44 px row showing the title and the line swatches. Rows for untitled stations say "untitled idea".
- **`↑`/`↓`/wheel** move the highlight. **`Enter`** or a click jumps to the highlighted result. **`Esc`** closes the box.
- **Jumping:**
  - centres the camera on the station, keeping the current zoom,
  - moves the lens focus to it,
  - selects it, which gives the Select energy bump,
  - then closes the box.
- If motion is allowed, animate the camera over ≤ 200 ms with ease-out. With **reduce motion** on, jump instantly. Request repaints only while the animation runs.
- **No matches:** show "Nothing matches yet". Use calm wording, with no counts.

### 7.2 Notes preview

- When the pointer has rested on a station for 400 ms, show a tooltip with the title and the first 3 lines of notes, cut off at 160 characters with "…".
- Show nothing if the notes are empty. Don't show it while dragging, while the picker, a menu, or search is open, or while renaming.
- Put the delay in a named `const`.

---

## 8. Docs and help (M7.7)

- Update the help overlay (`ui/help.rs`) and the README's interaction table with every new gesture:
  - right-drag to connect or branch,
  - right-click menus,
  - Shift+drag box select and Shift+click toggle,
  - group move,
  - `Ctrl+A`, `Ctrl+F`, `Esc`.
- Add an "M7" section to the README describing schema v2 and that v1 maps upgrade on load.

**PM walkthrough** (run the app and check each item):
1. Right-drag between two stations. The picker opens, the wheel scrolls the rows, and a click connects them. `Ctrl+Z` undoes it in one step.
2. Right-drag to empty space and pick a line. A new connected station appears with its rename box open. `Ctrl+Z` removes it in one step, including any new line.
3. Right-click a middle segment and choose **Remove connection**. The line splits in two. Undo restores it.
4. Shift+drag a box and drag the group. The ghost markers show where the stations will land, and one undo moves them all back.
5. Click empty space. The selection clears.
6. Switch palettes, set a custom colour, reorder lines, then check that the number keys follow the new order. Save, reopen, and confirm everything persisted. A v1 map opens fine.
7. `Ctrl+F`, type, then `Enter`. The view jumps to the station. With reduce motion on, it jumps instantly.
8. Hover a station with notes and the preview appears. No red appears anywhere, and every new target is at least 44 px.

---

## 9. Out of scope

- Drag-and-drop reordering of lines (the ▲/▼ buttons are enough for now).
- Branching lines, meaning a station with three or more segments on one line.
- Restricting custom colours.
