//! Opening, moving through, drawing and choosing from the line picker.

use super::MetroMindApp;
use crate::input::Command;
use crate::ui::picker::{Picker, Target, edit_for, rows, start_highlight, step};
use crate::ui::picker_view::{self, Action};
use eframe::egui::{Context, Rect};
use metromind_core::clock::Clock;
use metromind_core::energy::Touch;
use metromind_core::model::{Station, StationId};

impl MetroMindApp {
    pub(super) fn open_picker(&mut self, mut picker: Picker) {
        let rows = rows(&self.doc.map, &picker);
        picker.highlight = start_highlight(&self.doc.map, &picker, &rows);
        self.interaction.picker = Some(picker);
    }

    pub(super) fn picker_step(&mut self, delta: i32) {
        let Some(picker) = self.interaction.picker.as_mut() else {
            return;
        };
        let rows = rows(&self.doc.map, picker);
        picker.highlight = step(&rows, picker.highlight, delta);
    }

    /// Draw the picker, and queue a pick or cancel if the player made one.
    pub(super) fn show_picker(&mut self, ctx: &Context, viewport: Rect) {
        let Some(picker) = self.interaction.picker.as_mut() else {
            return;
        };
        let rows = rows(&self.doc.map, picker);
        let command = match picker_view::show(ctx, picker, &rows, viewport) {
            Some(Action::Pick(index)) => Command::PickerPick(index),
            Some(Action::Cancel) => Command::PickerCancel,
            None => return,
        };
        self.queued_commands.push(command);
    }

    /// Make the edit for row `index`, then select what it made. A refused row
    /// keeps the picker open and says why.
    pub(super) fn picker_pick(&mut self, index: usize) {
        let Some(picker) = self.interaction.picker.clone() else {
            return;
        };
        let rows = rows(&self.doc.map, &picker);
        let Some(row) = rows.get(index) else {
            return;
        };
        if let Some(reason) = &row.refusal {
            self.status.info(reason.clone());
            return;
        }
        let new_station = self.branch_station(&picker);
        let new_id = new_station.as_ref().map(|s| s.id);
        match edit_for(&self.doc.map, &picker, row.choice, new_station) {
            Ok(edit) => {
                self.interaction.picker = None;
                self.apply_edit(edit);
                self.touch(picker.origin, Touch::Connect);
                self.select_result(picker.target, new_id);
            }
            Err(refusal) => self.status.info(refusal.reason("")),
        }
    }

    /// The idea a branch will create, at the picked spot. `None` when connecting.
    fn branch_station(&self, picker: &Picker) -> Option<Station> {
        match picker.target {
            Target::Station(_) => None,
            Target::Spot(spot) => Some(Station::new(
                StationId(self.doc.map.peek_id()),
                "",
                spot,
                self.clock.now(),
                &self.settings.energy,
            )),
        }
    }

    /// Connecting selects the station joined to. Branching selects the new idea
    /// and opens its rename box straight away.
    fn select_result(&mut self, target: Target, new_id: Option<StationId>) {
        match (target, new_id) {
            (Target::Station(id), _) => self.select_only(Some(id)),
            (Target::Spot(_), Some(id)) => {
                self.select_only(Some(id));
                self.start_rename(id, String::new());
            }
            (Target::Spot(_), None) => {}
        }
    }
}
