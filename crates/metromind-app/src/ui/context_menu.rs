//! Right-click menus for stations, segments and empty space. Each item becomes
//! the same command as its keyboard shortcut.

use crate::input::{Action, Command};
use crate::render::colors::line_color;
use eframe::egui::{Button, RichText, Ui};
use metromind_core::Vec2;
use metromind_core::editing::{Membership, disconnect};
use metromind_core::model::{LineId, MetroMap, StationId};
use std::collections::BTreeSet;

/// Menu items are at least this tall, so they're easy to hit.
const MIN_TARGET: f32 = 44.0;

/// What a right-click landed on, captured at the moment of the click.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuTarget {
    Station(StationId),
    /// `insert_at` is where a new idea would go on the line.
    Segment {
        line: LineId,
        insert_at: usize,
        spot: Vec2,
    },
    Empty {
        spot: Vec2,
    },
}

/// What the menus read: the map and the current selection.
pub struct MenuData<'a> {
    pub map: &'a MetroMap,
    pub selection: &'a BTreeSet<StationId>,
}

pub fn show(ui: &mut Ui, target: MenuTarget, data: &MenuData, out: &mut Vec<Command>) {
    ui.spacing_mut().interact_size.y = MIN_TARGET;
    match target {
        MenuTarget::Station(id) => station_menu(ui, id, data, out),
        MenuTarget::Segment {
            line,
            insert_at,
            spot,
        } => segment_menu(ui, line, insert_at, spot, data, out),
        MenuTarget::Empty { spot } => empty_menu(ui, spot, out),
    }
}

fn station_menu(ui: &mut Ui, id: StationId, data: &MenuData, out: &mut Vec<Command>) {
    let count = data.selection.len();
    if count > 1 {
        ui.label(format!("{count} ideas selected"));
    }
    if item(ui, "Rename", "F2", true) {
        let text = data
            .map
            .stations
            .get(&id)
            .map(|s| s.title.clone())
            .unwrap_or_default();
        out.push(Command::StartRename { id, text });
    }
    if item(ui, "Notes", "Enter", true) {
        out.push(Command::OpenNotes(id));
    }
    ui.menu_button("Add to line", |ui| add_items(ui, data, out));
    ui.menu_button("Remove from line", |ui| remove_items(ui, data, out));
    if item(ui, "Start a new line", "N", true) {
        out.push(Command::Act(Action::NewLine(id)));
    }
    if item(ui, "Rekindle", "R", true) {
        out.push(Command::Act(Action::Rekindle));
    }
    if item(ui, "Delete", "Del", true) {
        out.push(Command::Act(Action::Delete));
    }
}

/// Every line. Lines where the whole selection is already greyed out.
fn add_items(ui: &mut Ui, data: &MenuData, out: &mut Vec<Command>) {
    for line in data.map.ordered_lines() {
        let all_on = data.selection.iter().all(|&id| line.contains(id));
        let label = line.name.as_str();
        if ui.add_enabled(!all_on, Button::new(label)).clicked() {
            out.push(Command::Act(Action::ToLine {
                line: line.id,
                remove: false,
            }));
        }
    }
}

/// Only the lines the selection is on.
fn remove_items(ui: &mut Ui, data: &MenuData, out: &mut Vec<Command>) {
    let mut any = false;
    for line in data.map.ordered_lines() {
        if !data.selection.iter().any(|&id| line.contains(id)) {
            continue;
        }
        any = true;
        if ui.button(line.name.as_str()).clicked() {
            out.push(Command::Act(Action::ToLine {
                line: line.id,
                remove: true,
            }));
        }
    }
    if !any {
        ui.add_enabled(false, Button::new("No lines to leave"));
    }
}

fn segment_menu(
    ui: &mut Ui,
    line_id: LineId,
    insert_at: usize,
    spot: Vec2,
    data: &MenuData,
    out: &mut Vec<Command>,
) {
    let Ok(line) = data.map.line(line_id) else {
        return;
    };
    ui.label(
        RichText::new(&line.name)
            .color(line_color(data.map, line))
            .strong(),
    );
    let index = insert_at.saturating_sub(1);
    let cut = disconnect(data.map, line_id, index);
    let mut remove = ui.add_enabled(cut.is_ok(), Button::new("Remove connection"));
    if let Err(refusal) = &cut {
        remove = remove.on_disabled_hover_text(refusal.reason(&line.name));
    }
    if remove.clicked() {
        out.push(Command::Act(Action::RemoveConnection {
            line: line_id,
            index,
        }));
    }
    if item(ui, "Highlight this line", "", true) {
        out.push(Command::HighlightLine(Some(line_id)));
    }
    if item(ui, "Insert idea here", "", true) {
        let membership = Membership {
            line: line_id,
            index: insert_at,
        };
        out.push(Command::NewIdea {
            spot,
            membership: Some(membership),
        });
    }
}

fn empty_menu(ui: &mut Ui, spot: Vec2, out: &mut Vec<Command>) {
    if item(ui, "New idea here", "", true) {
        out.push(Command::NewIdea {
            spot,
            membership: None,
        });
    }
    if item(ui, "Fit map", "Home", true) {
        out.push(Command::FitMap);
    }
    if item(ui, "Select all", "Ctrl+A", true) {
        out.push(Command::SelectAll);
    }
}

fn item(ui: &mut Ui, label: &str, shortcut: &str, enabled: bool) -> bool {
    let button = Button::new(label).shortcut_text(shortcut);
    ui.add_enabled(enabled, button).clicked()
}
