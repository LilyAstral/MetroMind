//! Right side panel for the selected idea: title, free-form notes, and the
//! lines it belongs to. Text is committed when a box loses focus, so typing
//! is one undo step rather than one per keystroke.

use crate::input::Command;
use crate::render::colors::line_color;
use crate::ui::note_draft::ListDraft;
use crate::ui::note_list;
use eframe::egui::{self, RichText, TextEdit, Ui};
use metromind_core::editing::Edit;
use metromind_core::energy::Touch;
use metromind_core::model::{MetroMap, NoteItem, StationId};

const PANEL_WIDTH: f32 = 320.0;
const NOTE_ROWS: usize = 14;

/// Draft text for the station shown in the panel.
#[derive(Debug, Clone, PartialEq)]
pub struct NotesState {
    pub station: StationId,
    pub title: String,
    pub notes: String,
    pub list: ListDraft,
}

#[derive(Debug, Default)]
pub struct NotesOutcome {
    pub edits: Vec<Edit>,
    pub commands: Vec<Command>,
    pub close: bool,
}

impl NotesState {
    pub fn new(map: &MetroMap, station: StationId) -> Option<Self> {
        let s = map.stations.get(&station)?;
        Some(Self {
            station,
            title: s.title.clone(),
            notes: s.notes.clone(),
            list: ListDraft::new(&s.items),
        })
    }

    /// Edits for any drafts that differ from the map (used on close/switch).
    pub fn pending_edits(&self, map: &MetroMap) -> Vec<Edit> {
        let Some(s) = map.stations.get(&self.station) else {
            return Vec::new();
        };
        let mut edits = Vec::new();
        if self.title != s.title {
            edits.push(Edit::Rename {
                id: self.station,
                title: self.title.clone(),
            });
        }
        if self.notes != s.notes {
            edits.push(Edit::SetNotes {
                id: self.station,
                notes: self.notes.clone(),
            });
        }
        let items: Vec<NoteItem> = self.list.cleaned();
        if items != s.items {
            edits.push(Edit::SetNoteItems {
                id: self.station,
                items,
            });
        }
        edits
    }
}

pub fn show(ui: &mut Ui, state: &mut NotesState, map: &MetroMap) -> NotesOutcome {
    let mut out = NotesOutcome::default();
    egui::Panel::right("notes_panel")
        .default_size(PANEL_WIDTH)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Idea");
                if ui.button("Close").on_hover_text("Esc").clicked() {
                    out.close = true;
                }
            });
            text_fields(ui, state, map, &mut out);
            list_field(ui, state, map, &mut out);
            lines_on(ui, state.station, map);
            actions(ui, state.station, &mut out);
        });
    out
}

fn text_fields(ui: &mut Ui, state: &mut NotesState, map: &MetroMap, out: &mut NotesOutcome) {
    let Some(station) = map.stations.get(&state.station) else {
        out.close = true;
        return;
    };
    let title = ui.add(
        TextEdit::singleline(&mut state.title)
            .hint_text("Name this idea")
            .align(egui::Align2::LEFT_CENTER),
    );
    sync(&title, &mut state.title, &station.title);
    ui.label("Notes");
    let notes = ui.add(
        TextEdit::multiline(&mut state.notes)
            .hint_text("Anything worth remembering…")
            .desired_rows(NOTE_ROWS)
            .desired_width(f32::INFINITY),
    );
    sync(&notes, &mut state.notes, &station.notes);
    if title.lost_focus() || notes.lost_focus() {
        out.edits.extend(state.pending_edits(map));
    }
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) && (title.has_focus() || notes.has_focus()) {
        out.close = true;
    }
}

/// While a box isn't being edited, mirror the model (e.g. after undo).
fn sync(response: &egui::Response, draft: &mut String, model: &str) {
    if !response.has_focus() && !response.lost_focus() && draft != model {
        model.clone_into(draft);
    }
}

fn list_field(ui: &mut Ui, state: &mut NotesState, map: &MetroMap, out: &mut NotesOutcome) {
    let Some(station) = map.stations.get(&state.station) else {
        return;
    };
    ui.add_space(8.0);
    note_list::show(
        ui,
        state.station,
        &station.items,
        station.list_style,
        &mut state.list,
        &mut out.edits,
    );
}

fn lines_on(ui: &mut Ui, station: StationId, map: &MetroMap) {
    ui.add_space(8.0);
    let lines: Vec<_> = map.lines_through(station).collect();
    if lines.is_empty() {
        ui.label(RichText::new("Not on a line yet. Press 1–9 or N.").weak());
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.label("On:");
        for line in lines {
            ui.label(
                RichText::new(&line.name)
                    .color(line_color(map, line))
                    .strong(),
            );
        }
    });
}

fn actions(ui: &mut Ui, station: StationId, out: &mut NotesOutcome) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui
            .button("Rekindle")
            .on_hover_text("R: give this idea a little warmth")
            .clicked()
        {
            out.commands.push(Command::Touch(station, Touch::Rekindle));
        }
        if ui
            .button("Delete idea")
            .on_hover_text("Del (Ctrl+Z brings it back)")
            .clicked()
        {
            out.edits.push(Edit::DeleteStation { id: station });
            out.commands.push(Command::Select(None));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use metromind_core::chrono::DateTime;
    use metromind_core::sample::sample_map;

    #[test]
    fn pending_edits_only_for_changes() {
        let map = sample_map(DateTime::UNIX_EPOCH);
        let id = *map.stations.keys().next().expect("stations");
        let mut state = NotesState::new(&map, id).expect("exists");
        assert!(state.pending_edits(&map).is_empty());
        state.notes = "new notes".into();
        assert_eq!(
            state.pending_edits(&map),
            vec![Edit::SetNotes {
                id,
                notes: "new notes".into()
            }]
        );
    }
}
