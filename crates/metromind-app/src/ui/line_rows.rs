//! The rows of the line list in the side panel. Left to right: colour (click
//! to pick a custom one), ↺ (back to the palette colour), number (click to
//! highlight the line), name, ▲/▼ (order), ✕ (delete). Hovering a row
//! highlights that line on the map.

use crate::input::Command;
use crate::render::colors::line_color;
use eframe::egui::{Align2, Button, Response, TextEdit, Ui, vec2};
use metromind_core::editing::Edit;
use metromind_core::model::{LineId, MetroMap};
use std::collections::BTreeMap;

/// Every row button meets this hit target, even when its icon is smaller.
const TARGET: f32 = 44.0;
/// Narrowest the name field may get.
const MIN_NAME_WIDTH: f32 = 60.0;
/// Six buttons per row, and the gaps between them.
const ROW_BUTTONS: usize = 6;

/// Text buffers for line names being edited, keyed by line.
pub type LineNameDrafts = BTreeMap<LineId, String>;

pub fn show(
    ui: &mut Ui,
    map: &MetroMap,
    locked: Option<LineId>,
    drafts: &mut LineNameDrafts,
    edits: &mut Vec<Edit>,
    commands: &mut Vec<Command>,
) {
    let last = map.line_order.len().saturating_sub(1);
    let mut hovered = None;
    for (index, line) in map.ordered_lines().enumerate() {
        let row = ui.horizontal(|ui| {
            let name_width = name_width(ui.available_width());
            colour_controls(ui, map, line.id, edits);
            number_button(ui, index, line.id, locked, commands);
            name_field(ui, line.id, &line.name, name_width, drafts, edits);
            let up = index > 0;
            if order_button(ui, "▲", up, "Move up").clicked() {
                edits.push(Edit::MoveLine {
                    id: line.id,
                    to: index - 1,
                });
            }
            let down = index < last;
            if order_button(ui, "▼", down, "Move down").clicked() {
                edits.push(Edit::MoveLine {
                    id: line.id,
                    to: index + 1,
                });
            }
            if ui
                .add(Button::new("✕").min_size(vec2(TARGET, TARGET)))
                .on_hover_text("Delete this line (Ctrl+Z brings it back)")
                .clicked()
            {
                edits.push(Edit::DeleteLine { id: line.id });
            }
        });
        if row.response.hovered() {
            hovered = Some(line.id);
        }
    }
    commands.push(Command::HoverLine(hovered));
}

fn name_width(available: f32) -> f32 {
    let spacing = ROW_BUTTONS as f32 * 4.0;
    (available - TARGET * ROW_BUTTONS as f32 - spacing).max(MIN_NAME_WIDTH)
}

/// The swatch opens egui's colour picker; ↺ returns to the palette colour.
fn colour_controls(ui: &mut Ui, map: &MetroMap, id: LineId, edits: &mut Vec<Edit>) {
    let Ok(line) = map.line(id) else {
        return;
    };
    let current = line_color(map, line);
    let mut rgb = [current.r(), current.g(), current.b()];
    let response = ui
        .scope(|ui| {
            ui.spacing_mut().interact_size = vec2(TARGET, TARGET);
            ui.color_edit_button_srgb(&mut rgb)
        })
        .inner
        .on_hover_text("Change this line's colour");
    if response.changed() {
        edits.push(Edit::SetLineColor {
            id,
            custom: Some(rgb),
        });
    }
    let customised = line.custom_color.is_some();
    let reset = ui
        .add_enabled(customised, Button::new("↺").min_size(vec2(TARGET, TARGET)))
        .on_hover_text("Use the palette colour");
    if reset.clicked() {
        edits.push(Edit::SetLineColor { id, custom: None });
    }
}

/// Clicking the number highlights this line; clicking it again clears it.
fn number_button(
    ui: &mut Ui,
    index: usize,
    id: LineId,
    locked: Option<LineId>,
    commands: &mut Vec<Command>,
) {
    let on = locked == Some(id);
    let number = Button::new(format!("{}", index + 1))
        .selected(on)
        .min_size(vec2(TARGET, TARGET));
    if ui
        .add(number)
        .on_hover_text("Highlight this line")
        .clicked()
    {
        let next = if on { None } else { Some(id) };
        commands.push(Command::HighlightLine(next));
    }
}

fn name_field(
    ui: &mut Ui,
    id: LineId,
    name: &str,
    width: f32,
    drafts: &mut LineNameDrafts,
    edits: &mut Vec<Edit>,
) {
    let draft = drafts.entry(id).or_insert_with(|| name.to_owned());
    let response = ui.add(
        TextEdit::singleline(draft)
            .desired_width(width)
            .min_size(vec2(width, TARGET))
            .align(Align2::LEFT_CENTER),
    );
    if response.lost_focus() {
        if !draft.trim().is_empty() && *draft != name {
            edits.push(Edit::RenameLine {
                id,
                name: draft.trim().to_owned(),
            });
        }
        drafts.remove(&id);
    } else if !response.has_focus() {
        *draft = name.to_owned();
    }
}

fn order_button(ui: &mut Ui, label: &str, enabled: bool, hover: &str) -> Response {
    ui.add_enabled(enabled, Button::new(label).min_size(vec2(TARGET, TARGET)))
        .on_hover_text(hover)
}
