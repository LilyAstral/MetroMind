//! The list under a station's notes, as bullets or checkboxes. Enter adds an
//! item, Backspace on an empty item removes it, and Alt+↑/↓ moves it. Item
//! text is a draft that commits when the list loses focus; ticks, adds,
//! removals and moves commit at once. Ticked items are dimmed, never counted.

use super::note_draft::{ListDraft, RowAction};
use crate::render::colors::Theme;
use eframe::egui::text::CCursor;
use eframe::egui::text_selection::CCursorRange;
use eframe::egui::{self, Align2, Button, Id, Key, Modifiers, TextEdit, Ui, vec2};
use metromind_core::editing::Edit;
use metromind_core::model::{ListStyle, NoteItem, StationId};

/// Every row control meets this hit target.
const TARGET: f32 = 44.0;
/// Fixed widths, never derived from the panel's width (see `ui/line_rows.rs`).
const TEXT_WIDTH: f32 = 200.0;
const STYLE_WIDTH: f32 = 140.0;

/// Draw the style switch, the rows and the add button. Edits go to `edits`.
pub fn show(
    ui: &mut Ui,
    station: StationId,
    saved: &[NoteItem],
    style: ListStyle,
    draft: &mut ListDraft,
    edits: &mut Vec<Edit>,
) {
    draft.sync(saved);
    style_switch(ui, station, style, edits);
    let base = ui.make_persistent_id(("note_items", station.0));
    let action = rows(ui, base, draft, style);
    let mut committed = false;
    if let Some(action) = action {
        draft.apply(action);
        committed = commit(station, draft, saved, edits);
    }
    let requested = request_focus(ui, base, draft);
    let now_focused =
        requested || (0..draft.items.len()).any(|i| ui.memory(|m| m.has_focus(base.with(i))));
    if draft.focused && !now_focused {
        commit_on_blur(station, draft, saved, committed, edits);
    }
    draft.focused = now_focused;
}

/// One row per item, then the add button. Returns the single edit the user
/// asked for this frame, if any.
fn rows(ui: &mut Ui, base: Id, draft: &mut ListDraft, style: ListStyle) -> Option<RowAction> {
    let len = draft.items.len();
    let reveal = draft.reveal.take();
    let mut action = None;
    for (index, item) in draft.items.iter_mut().enumerate() {
        let row_id = base.with(index);
        let has_focus = ui.memory(|m| m.has_focus(row_id));
        let keys = key_action(ui, has_focus, index, len, item);
        let row = item_row(ui, row_id, index, item, style, reveal == Some(index));
        action = action.or(keys).or(row);
    }
    let add = Button::new("+ Add item").min_size(vec2(TEXT_WIDTH, TARGET));
    if ui.add(add).clicked() {
        action = action.or(Some(RowAction::Insert(len)));
    }
    action
}

/// Drop blank items when the list loses focus. If this frame already queued
/// a commit, rewrite that edit rather than queueing a second undo step.
fn commit_on_blur(
    station: StationId,
    draft: &mut ListDraft,
    saved: &[NoteItem],
    committed: bool,
    edits: &mut Vec<Edit>,
) {
    draft.items = draft.cleaned();
    if !committed {
        commit(station, draft, saved, edits);
    } else if let Some(Edit::SetNoteItems { items, .. }) = edits.last_mut() {
        *items = draft.items.clone();
    }
}

/// Queue the whole draft if it differs from the saved list.
fn commit(
    station: StationId,
    draft: &ListDraft,
    saved: &[NoteItem],
    edits: &mut Vec<Edit>,
) -> bool {
    if draft.items == saved {
        return false;
    }
    edits.push(Edit::SetNoteItems {
        id: station,
        items: draft.items.clone(),
    });
    true
}

fn style_switch(ui: &mut Ui, station: StationId, style: ListStyle, edits: &mut Vec<Edit>) {
    ui.horizontal(|ui| {
        let choices = [
            (ListStyle::Bullets, "• Bullets"),
            (ListStyle::Checkboxes, "☐ Checkboxes"),
        ];
        for (choice, label) in choices {
            let button = Button::new(label)
                .selected(style == choice)
                .min_size(vec2(STYLE_WIDTH, TARGET));
            if ui.add(button).clicked() && style != choice {
                edits.push(Edit::SetListStyle {
                    id: station,
                    style: choice,
                });
            }
        }
    });
}

/// Keyboard edits for the focused row. Keys are consumed so the text box
/// never sees them.
fn key_action(
    ui: &Ui,
    has_focus: bool,
    index: usize,
    len: usize,
    item: &NoteItem,
) -> Option<RowAction> {
    if !has_focus {
        return None;
    }
    ui.input_mut(|input| {
        if input.consume_key(Modifiers::ALT, Key::ArrowUp) {
            return (index > 0).then(|| RowAction::Move {
                from: index,
                to: index - 1,
            });
        }
        if input.consume_key(Modifiers::ALT, Key::ArrowDown) {
            return (index + 1 < len).then_some(RowAction::Move {
                from: index,
                to: index + 1,
            });
        }
        if input.consume_key(Modifiers::NONE, Key::Enter) {
            return Some(RowAction::Insert(index + 1));
        }
        if item.text.is_empty() && input.consume_key(Modifiers::NONE, Key::Backspace) {
            return Some(RowAction::Remove(index));
        }
        None
    })
}

fn item_row(
    ui: &mut Ui,
    row_id: Id,
    index: usize,
    item: &mut NoteItem,
    style: ListStyle,
    reveal: bool,
) -> Option<RowAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        action = marker(ui, index, item.done, style);
        let mut text = TextEdit::singleline(&mut item.text)
            .id(row_id)
            .desired_width(TEXT_WIDTH)
            .align(Align2::LEFT_CENTER);
        if style == ListStyle::Checkboxes && item.done {
            text = text.text_color(Theme::default().text_dim);
        }
        let response = ui.add(text);
        if reveal {
            response.scroll_to_me(None);
        }
        let remove = Button::new("✕").min_size(vec2(TARGET, TARGET));
        if ui.add(remove).on_hover_text("Remove this item").clicked() {
            action = Some(RowAction::Delete(index));
        }
    });
    action
}

/// The bullet (view only) or the checkbox (ticks the item).
fn marker(ui: &mut Ui, index: usize, done: bool, style: ListStyle) -> Option<RowAction> {
    match style {
        ListStyle::Bullets => {
            ui.add_sized(vec2(TARGET, TARGET), egui::Label::new("•"));
            None
        }
        ListStyle::Checkboxes => {
            let glyph = if done { "☑" } else { "☐" };
            let tick = Button::new(glyph).min_size(vec2(TARGET, TARGET));
            ui.add(tick)
                .on_hover_text("Tick off")
                .clicked()
                .then_some(RowAction::Tick(index))
        }
    }
}

/// Move focus to the row the last action asked for. Returns whether it did.
fn request_focus(ui: &Ui, base: Id, draft: &mut ListDraft) -> bool {
    let Some(index) = draft.focus.take() else {
        return false;
    };
    let id = base.with(index);
    let end = draft
        .items
        .get(index)
        .map_or(0, |item| item.text.chars().count());
    let mut state = TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
    state
        .cursor
        .set_char_range(Some(CCursorRange::one(CCursor::new(end))));
    TextEdit::store_state(ui.ctx(), id, state);
    ui.memory_mut(|m| m.request_focus(id));
    draft.reveal = Some(index);
    true
}
