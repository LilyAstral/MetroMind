//! Selection: click, Shift+click, box select, Ctrl+A, and keeping the selection
//! in step with the map after edits.

use super::MetroMindApp;
use metromind_core::energy::Touch;
use metromind_core::model::StationId;
use std::collections::BTreeSet;

impl MetroMindApp {
    /// Replace the selection with one station, or clear it.
    pub(super) fn select_only(&mut self, id: Option<StationId>) {
        self.set_selection(id.into_iter().collect(), id);
    }

    /// Shift+click: add the station to the selection, or remove it.
    pub(super) fn toggle(&mut self, id: StationId) {
        let mut selection = self.interaction.selection.clone();
        let primary = if selection.remove(&id) {
            match self.interaction.primary {
                Some(p) if p != id => Some(p),
                _ => selection.last().copied(),
            }
        } else {
            selection.insert(id);
            Some(id)
        };
        self.set_selection(selection, primary);
    }

    pub(super) fn add_to_selection(&mut self, ids: Vec<StationId>) {
        if ids.is_empty() {
            return;
        }
        let mut selection = self.interaction.selection.clone();
        selection.extend(ids.iter().copied());
        self.set_selection(selection, ids.last().copied());
    }

    pub(super) fn select_all(&mut self) {
        let selection: BTreeSet<StationId> = self.doc.map.stations.keys().copied().collect();
        let primary = self
            .interaction
            .primary
            .filter(|p| selection.contains(p))
            .or_else(|| selection.first().copied());
        self.set_selection(selection, primary);
    }

    /// Selecting counts as a little attention for a newly focused station.
    /// The notes pane follows the primary station.
    fn set_selection(&mut self, selection: BTreeSet<StationId>, primary: Option<StationId>) {
        if self.rename.as_ref().is_some_and(|r| Some(r.id) != primary) {
            self.finish_rename(true);
        }
        let changed = primary != self.interaction.primary;
        self.interaction.selection = selection;
        self.interaction.primary = primary;
        match primary {
            Some(id) if changed => {
                self.touch(id, Touch::Select);
                if self.notes.is_some() {
                    self.open_notes(id);
                }
            }
            Some(_) => {}
            None => self.close_notes(),
        }
    }
}
