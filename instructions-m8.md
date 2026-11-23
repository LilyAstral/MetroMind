# MetroMind M8: Glass hover panel and note lists

> Follow-on to `instructions.md` and `instructions-m7.md`. Everything in those still applies, especially `instructions.md` §7 (architecture), §8 (code standards), and §12 (ADHD checklist). Where this file changes something in them, this file wins.

---

## 0. Working agreement

**Mode: LOOSE inside this spec.** Work through the milestones in order without waiting for approval at each one. Stop and report if a feature would need a new crate, can only be built with tangled code, or conflicts with the earlier specs in a way this file doesn't settle.

**At the end of each milestone:**
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.
- Commit as `M8.x: <summary>`.
- Work on a branch, not `master`.

**Lesson from M7, which applies to every panel and popup you touch:** never size a widget from `ui.available_width()` inside a panel or `Area` that grows to fit its content. That creates a feedback loop, and the panel spreads across the window. Use fixed widths taken from named `const`s, and cap panels with `size_range`.

---

## 1. Summary

| # | Feature | Milestone |
|---|---|---|
| A | Each station gets a list of items that can be added to. The list can be shown as bullets or as checkboxes. | M8.1 (core), M8.2 (notes panel) |
| B | Hovering a station shows its title, notes, and list in a sleek glass panel to the right of the cursor. This replaces the M7 tooltip. | M8.3 |

---

## 2. Core: note lists (M8.1)

All of this lives in `metromind-core`, with unit tests.

### 2.1 Model

```rust
pub struct NoteItem {
    pub text: String,
    pub done: bool,
}

pub enum ListStyle {
    #[default]
    Bullets,
    Checkboxes,
}

// added to Station
#[serde(default)] pub items: Vec<NoteItem>,
#[serde(default)] pub list_style: ListStyle,
```

- The free-form `notes` text stays. The list sits **below** it, as an addition, not a replacement.
- **Switching styles keeps each item's `done` flag.** Going from Checkboxes to Bullets and back restores the ticks. Bullets mode just doesn't show them.

### 2.2 Edits

There are two new `Edit` variants. Both are undoable and both bump energy as `Touch::Edit`.
- `SetNoteItems { id, items: Vec<NoteItem> }` replaces the whole list and returns the old list as its inverse. This matches how `SetNotes` replaces the whole text. Adding, removing, editing, reordering, and ticking items all go through it.
- `SetListStyle { id, style }`

`DeleteStation` already restores the whole `Station` on undo, so the list comes back with it. Add a test that proves this.

### 2.3 Schema v3

- Change `SCHEMA_VERSION` to 3. Files at v1 and v2 still load, with an empty list in Bullets style.
- Extend `MetroMap::upgrade()`. Don't add a second migration path.
- **Tests:**
  - a v2 fixture loads with empty lists,
  - a v3 save→load round trip is exact,
  - v4 is refused with the existing friendly message.

### 2.4 Search

`core::search` also matches item text. It ranks below title matches and alongside notes matches.

---

## 3. Notes panel list editor (M8.2)

Add this to `ui/notes_panel.rs`, under the Notes box. Move it into its own file, `ui/note_list.rs`, if the panel grows past about 200 lines.

- **Style switch:** a two-option segmented control, **"• Bullets"** and **"☐ Checkboxes"**. It sends `SetListStyle`.
- **One row per item:**
  - In Bullets mode, a bullet. In Checkboxes mode, a checkbox; clicking it sends `SetNoteItems` with that item's `done` flipped.
  - A single-line `TextEdit` with a fixed width, set by a `const`.
  - A ✕ button that removes the item.
  - Every row target is at least 44 px.
- **Adding items:**
  - A **"+ Add item"** button under the list appends an empty item and gives it focus.
  - Pressing **`Enter`** in an item inserts a new empty item below it and moves focus there. This makes it quick to type a list one item after another.
- **Removing with Backspace:** pressing **`Backspace` in an empty item** removes it and moves focus to the end of the item above. Nothing happens if it's the only item.
- **Reordering:** **`Alt+↑` / `Alt+↓`** in an item move it up or down. No drag-and-drop.
- **Committing changes:**
  - Item text is committed the same way as the notes text: one `SetNoteItems` edit when focus leaves the list, not one per keystroke. Keep the list draft in `NotesState` and add it to `pending_edits()`.
  - Ticking, removing, adding, and moving items commit right away.
  - An empty item left behind when focus leaves the list is dropped as part of that commit.
- **Ticked items** show their text dimmed, using `text_dim`. They stay where they are. Don't use strikethrough, don't sort ticked items to the bottom, and **show no counts** such as "3/5 done" anywhere, per the §12 no-shame rule.

---

## 4. Glass hover panel (M8.3)

This replaces the M7 tooltip in `ui/hover_preview.rs`. Rename the file to `ui/hover_panel.rs`.

### 4.1 When it shows

- It appears after the pointer rests on a station for `HOVER_DELAY` (keep it at 0.4 s).
- It shows only if the station has notes **or** list items.
- It's hidden while you drag, while the picker, a context menu, or search is open, and while renaming. These are the same rules as M7.
- **Moving the pointer:** while the pointer stays on the same station, the panel follows the cursor. Moving to another station swaps the panel without waiting for the delay again. Leaving all stations hides it.
- The panel is not interactive. Use `.interactable(false)` so it never steals hover or clicks from the map.

### 4.2 Position

- Its top-left corner is **16 px right of the cursor and 8 px above it**, so the panel sits beside the cursor and doesn't cover the station under it.
- If the panel would run past the right edge of the map viewport, flip it to the **left** of the cursor, with the same gap.
- Clamp it vertically so it stays inside the viewport.
- Keep the placement maths in a pure function, `fn place(cursor, panel_size, viewport) -> Pos2`, with unit tests for the normal case, the flip to the left, and the vertical clamp.

### 4.3 Look: "sleek glass"

egui can't blur what's behind a panel, so build the glass look from layers in one helper, `ui/glass.rs::glass_frame(painter, rect)`, and reuse it later for other popups:

1. **Soft shadow:** an offset, blurred `Shadow` with a low alpha.
2. **Body:** a rounded rectangle with a 14 px corner radius, filled with the theme background at about 70% alpha, so the map shows faintly through it.
3. **Sheen:** a vertical gradient mesh over the top third, from white at about 8% alpha down to 0%. Build it like `radial_glow`, using vertex colours.
4. **Edge:** a 1 px border in white at about 14% alpha, with a slightly brighter top edge.
5. **Optional:** the existing grain texture at a very low alpha, so the glass picks up the map's texture.

Put every alpha, radius, and offset in a named `const` at the top of `glass.rs`.

### 4.4 Content

- **Fixed width:** 280 px.
- **Title:** strong text, with small swatches for the lines the station is on.
- **Notes:** the first 4 lines, cut at 200 characters with "…". Reuse and adjust `preview_text`.
- **List:** up to 6 items, each shown as a bullet or a read-only checkbox (☐ / ☑ glyphs) to match the station's list style. Ticked items are dimmed. If there are more items, end with a single "…" line, **not** a count.
- All text must keep at least 3:1 contrast against the glass at its most transparent. Add a test that checks `text` and `text_dim` against the glass colour blended over the map background.

### 4.5 Motion

- **Fade-in:** over 120 ms, from 0 to full alpha, using egui's `animate_bool_with_time`. Repaint only while the fade runs.
- **Reduce motion on:** the panel appears instantly.
- **Hiding:** instant, with no fade-out. A panel that lingers after the pointer has left feels laggy.

---

## 5. Docs (M8.4)

Update the README and the help overlay to cover:
- the note list,
- `Enter` / `Backspace` / `Alt+↑↓` in list items,
- the Bullets/Checkboxes switch,
- the hover panel.

Add a short note about schema v3 to the README.

**PM walkthrough:**
1. Hover a station that has notes. After a short pause, a glass panel appears to the right of the cursor and follows it. Near the right edge of the window, it flips to the left of the cursor.
2. In the notes pane:
   - Type a list using `Enter` between items.
   - Press `Backspace` on an empty item to remove it.
   - Switch to Checkboxes and tick two items.
   - Switch back to Bullets, then to Checkboxes again. The ticks are still there.
3. Hover the station again. The panel shows the checkboxes, with ticked items dimmed and no counts.
4. `Ctrl+Z` steps back through the list changes. Deleting the station and undoing brings the list back.
5. Save, reopen, and check the list persists. An M7 (v2) map opens fine.
6. With reduce motion on, the panel appears without fading.

---

## 6. Out of scope

- Nested sub-items or indentation.
- Real background blur. It would need a custom render pass, and the layered glass in §4.3 stands in for it.
- Clicking or ticking items inside the hover panel. The panel is view-only, and editing happens in the notes pane.
- Rich text or Markdown in notes.
