//! Inline rename box that floats right at the station.

use eframe::egui::{
    self, Area, Context, Id, Key, Order, Pos2, TextEdit,
    text::{CCursor, CCursorRange},
};
use metromind_core::model::StationId;

const BOX_WIDTH: f32 = 240.0;

#[derive(Debug, Clone, PartialEq)]
pub struct RenameState {
    pub id: StationId,
    pub text: String,
    /// First frame: grab focus and put the cursor at the end.
    pub fresh: bool,
}

impl RenameState {
    pub fn new(id: StationId, text: String) -> Self {
        Self {
            id,
            text,
            fresh: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameResult {
    Editing,
    Commit,
    Cancel,
}

/// Enter or clicking away commits; Esc cancels.
pub fn show(ctx: &Context, state: &mut RenameState, at: Pos2) -> RenameResult {
    let area = Area::new(Id::unique("rename_station"))
        .order(Order::Foreground)
        .fixed_pos(at);
    area.show(ctx, |ui| {
        egui::Frame::popup(ui.style())
            .show(ui, |ui| edit_box(ui, state))
            .inner
    })
    .inner
}

fn edit_box(ui: &mut egui::Ui, state: &mut RenameState) -> RenameResult {
    let mut output = TextEdit::singleline(&mut state.text)
        .hint_text("Name this idea")
        .desired_width(BOX_WIDTH)
        .align(egui::Align2::LEFT_CENTER)
        .show(ui);
    if state.fresh {
        state.fresh = false;
        output.response.request_focus();
        let end = CCursor::new(state.text.chars().count());
        output
            .state
            .cursor
            .set_char_range(Some(CCursorRange::one(end)));
        output.state.store(ui.ctx(), output.response.id);
        return RenameResult::Editing;
    }
    if output.response.lost_focus() {
        if ui.input(|i| i.key_pressed(Key::Escape)) {
            RenameResult::Cancel
        } else {
            RenameResult::Commit
        }
    } else {
        RenameResult::Editing
    }
}
