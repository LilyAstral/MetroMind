//! Applying view-level [`Command`]s and managing selection, rename and notes.

use super::MetroMindApp;
use crate::frame_plan::to_vec2;
use crate::input::Command;
use crate::ui::{notes_panel::NotesState, rename::RenameState};
use eframe::egui::Rect;
use metromind_core::Vec2;
use metromind_core::editing::Edit;
use metromind_core::energy::Touch;
use metromind_core::geometry::Camera;
use metromind_core::model::StationId;

/// Empty space kept around the map by "Fit whole map".
const FIT_MARGIN: f32 = 80.0;

impl MetroMindApp {
    pub(super) fn run_command(&mut self, command: Command, viewport: Rect) {
        match command {
            Command::Select(id) => self.select(id),
            Command::Touch(id, kind) => self.touch(id, kind),
            Command::Drag(drag) => self.interaction.drag = drag,
            Command::Pan(delta) => self.settings.camera = self.settings.camera.panned(delta),
            Command::Zoom { anchor, factor } => {
                self.settings.camera = self.settings.camera.zoomed_around(anchor, factor);
            }
            Command::AdjustLensStrength(d) => {
                self.settings.lens = self
                    .settings
                    .lens
                    .with_strength(self.settings.lens.strength + d);
            }
            Command::AdjustLensRadius(d) => {
                self.settings.lens = self
                    .settings
                    .lens
                    .with_radius(self.settings.lens.radius + d);
            }
            Command::ToggleLens => self.toggle_lens(),
            Command::FitMap => self.fit(viewport),
            Command::StartRename { id, text } => self.start_rename(id, text),
            Command::OpenNotes(id) => self.open_notes(id),
            Command::Undo => self.undo(),
            Command::Redo => self.redo(),
            Command::NewMap => self.new_map(viewport),
            Command::Open => self.open_dialog(viewport),
            Command::Save => self.save_now(),
            Command::SaveAs => self.save_as(),
            Command::LoadSample => self.load_sample(viewport),
            Command::ToggleHelp => self.show_help = !self.show_help,
            Command::Notice(text) => self.status.info(text),
        }
    }

    fn touch(&mut self, id: StationId, kind: Touch) {
        if let Err(err) = self.doc.touch(id, kind, &self.clock, &self.settings.energy) {
            self.status.problem(err.to_string());
        }
    }

    /// Selecting counts as a little attention. The notes pane follows the selection.
    pub(super) fn select(&mut self, id: Option<StationId>) {
        if self.interaction.selected == id {
            return;
        }
        if self.rename.as_ref().is_some_and(|r| Some(r.id) != id) {
            self.finish_rename(true);
        }
        self.interaction.selected = id;
        match id {
            Some(id) => {
                self.touch(id, Touch::Select);
                if self.notes.is_some() {
                    self.open_notes(id);
                }
            }
            None => self.close_notes(),
        }
    }

    fn toggle_lens(&mut self) {
        self.settings.lens_enabled = !self.settings.lens_enabled;
        let text = if self.settings.lens_enabled {
            "Lens on"
        } else {
            "Lens off · L brings it back"
        };
        self.status.info(text);
    }

    /// Show the whole map. An empty map centers the origin.
    pub(super) fn fit(&mut self, viewport: Rect) {
        let (lo, hi) = self.doc.map.bounds().unwrap_or((Vec2::ZERO, Vec2::ZERO));
        let size = to_vec2(viewport.max) - to_vec2(viewport.min);
        self.settings.camera = Camera::fit(lo, hi, to_vec2(viewport.min), size, FIT_MARGIN);
    }

    fn start_rename(&mut self, id: StationId, text: String) {
        if self.rename.is_some() {
            self.finish_rename(true);
        }
        self.rename = Some(RenameState::new(id, text));
    }

    pub(super) fn finish_rename(&mut self, commit: bool) {
        let Some(state) = self.rename.take() else {
            return;
        };
        let current = self
            .doc
            .map
            .stations
            .get(&state.id)
            .map(|s| s.title.as_str());
        let title = state.text.trim().to_owned();
        if commit && current.is_some_and(|t| t != title) {
            self.apply_edit(Edit::Rename {
                id: state.id,
                title,
            });
        }
    }

    fn open_notes(&mut self, id: StationId) {
        self.flush_notes();
        self.notes = NotesState::new(&self.doc.map, id);
    }

    pub(super) fn close_notes(&mut self) {
        self.flush_notes();
        self.notes = None;
    }

    fn flush_notes(&mut self) {
        let edits = self.notes.as_ref().map(|n| n.pending_edits(&self.doc.map));
        for edit in edits.unwrap_or_default() {
            self.apply_edit(edit);
        }
    }

    /// Commit any half-typed text before saving or switching files.
    pub(super) fn flush_drafts(&mut self) {
        self.finish_rename(true);
        self.flush_notes();
    }

    fn undo(&mut self) {
        self.flush_drafts();
        match self.doc.undo() {
            Ok(true) => {}
            Ok(false) => self.status.info("Nothing to undo"),
            Err(err) => self.status.problem(err.to_string()),
        }
        self.forget_missing_selection();
    }

    fn redo(&mut self) {
        match self.doc.redo() {
            Ok(true) => {}
            Ok(false) => self.status.info("Nothing to redo"),
            Err(err) => self.status.problem(err.to_string()),
        }
        self.forget_missing_selection();
    }

    /// Undo or delete can remove the selected station; drop stale references.
    pub(super) fn forget_missing_selection(&mut self) {
        let exists = |id: &StationId| self.doc.map.stations.contains_key(id);
        if self.interaction.selected.is_some_and(|id| !exists(&id)) {
            self.interaction.selected = None;
        }
        if self.notes.as_ref().is_some_and(|n| !exists(&n.station)) {
            self.notes = None;
        }
        if self.rename.as_ref().is_some_and(|r| !exists(&r.id)) {
            self.rename = None;
        }
        if self.interaction.drag.is_some_and(|d| !exists(&d.id)) {
            self.interaction.drag = None;
        }
    }
}
