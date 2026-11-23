//! Doing things to the selection: number keys, Del, R, N, double-click and
//! the context menus all end up here.

use super::MetroMindApp;
use crate::input::Action;
use metromind_core::Vec2;
use metromind_core::clock::Clock;
use metromind_core::editing::{Edit, Membership, disconnect};
use metromind_core::energy::Touch;
use metromind_core::model::{LINE_COLOR_COUNT, Line, LineId, Station, StationId};
use std::collections::BTreeSet;

impl MetroMindApp {
    pub(super) fn run_action(&mut self, action: Action) {
        match action {
            Action::ToLine { line, remove } => self.act_on_line(line, remove),
            Action::NewLine(station) => self.new_line(station),
            Action::Rekindle => self.rekindle_selection(),
            Action::Delete => self.delete_selection(),
            Action::RemoveConnection { line, index } => self.remove_connection(line, index),
        }
    }

    /// Create an idea at `spot`, optionally on a segment, and start naming it.
    pub(super) fn new_idea(&mut self, spot: Vec2, membership: Option<Membership>) {
        let station = Station::new(
            StationId(self.doc.map.peek_id()),
            "",
            spot,
            self.clock.now(),
            &self.settings.energy,
        );
        let id = station.id;
        let memberships = membership.into_iter().collect();
        self.apply_edit(Edit::AddStation {
            station,
            memberships,
        });
        self.select_only(Some(id));
        self.start_rename(id, String::new());
    }

    fn act_on_line(&mut self, id: LineId, remove: bool) {
        let Ok(line) = self.doc.map.line(id).cloned() else {
            return;
        };
        let selection = self.interaction.selection.clone();
        let edits = line_edits(&line, remove, &selection);
        if edits.is_empty() {
            let notice = skipped_notice(&line, remove, selection.len());
            self.status.info(notice);
        } else {
            self.apply_edit(Edit::Batch(edits));
        }
    }

    /// Start a new line, with `station` as its first idea.
    fn new_line(&mut self, station: StationId) {
        let map = &self.doc.map;
        if map.lines.len() >= usize::from(LINE_COLOR_COUNT) {
            let hint = "All 9 lines are in use. Add this idea to one with 1–9.";
            self.status.info(hint);
            return;
        }
        let line = Line {
            id: LineId(map.peek_id()),
            name: format!("Line {}", map.lines.len() + 1),
            color_index: map.free_color_index().unwrap_or(0),
            stations: vec![station],
            custom_color: None,
        };
        let hint = format!("Started {}. Rename it in the side panel.", line.name);
        let position = map.line_order.len();
        self.apply_edit(Edit::CreateLine { line, position });
        self.status.info(hint);
    }

    fn rekindle_selection(&mut self) {
        let selection = self.interaction.selection.clone();
        for &id in &selection {
            self.touch(id, Touch::Rekindle);
        }
        let notice = match (selection.len(), self.interaction.primary) {
            (1, Some(id)) => self
                .doc
                .map
                .stations
                .get(&id)
                .map(|s| format!("Rekindled “{}”", s.display_title()))
                .unwrap_or_default(),
            (n, _) => format!("Rekindled {n} ideas"),
        };
        self.status.info(notice);
    }

    fn delete_selection(&mut self) {
        let selection = self.interaction.selection.clone();
        let count = selection.len();
        let edits = selection
            .into_iter()
            .map(|id| Edit::DeleteStation { id })
            .collect();
        self.apply_edit(Edit::Batch(edits));
        self.select_only(None);
        let notice = match count {
            1 => "Idea removed. Ctrl+Z brings it back.".to_owned(),
            _ => format!("{count} ideas removed. Ctrl+Z brings them back."),
        };
        self.status.info(notice);
    }

    fn remove_connection(&mut self, line: LineId, index: usize) {
        match disconnect(&self.doc.map, line, index) {
            Ok(edit) => self.apply_edit(edit),
            Err(refusal) => {
                let name = self
                    .doc
                    .map
                    .line(line)
                    .map(|l| l.name.clone())
                    .unwrap_or_default();
                self.status.info(refusal.reason(&name));
            }
        }
    }
}

/// Edits that add each selected station to `line` (or take it off, with
/// `remove`). Stations where the action does nothing are skipped. Removals go
/// from the highest index down so the earlier ones stay valid.
fn line_edits(line: &Line, remove: bool, selection: &BTreeSet<StationId>) -> Vec<Edit> {
    if remove {
        let mut indexes: Vec<usize> = selection
            .iter()
            .filter_map(|&id| line.index_of(id))
            .collect();
        indexes.sort_unstable_by(|a, b| b.cmp(a));
        return indexes
            .into_iter()
            .map(|index| Edit::RemoveFromLine {
                line: line.id,
                index,
            })
            .collect();
    }
    let mut next = line.stations.len();
    selection
        .iter()
        .filter(|&&id| !line.contains(id))
        .map(|&station| {
            let edit = Edit::AddToLine {
                line: line.id,
                station,
                index: next,
            };
            next += 1;
            edit
        })
        .collect()
}

fn skipped_notice(line: &Line, remove: bool, count: usize) -> String {
    match (remove, count) {
        (true, 1) => format!("This idea isn't on {}.", line.name),
        (true, _) => format!("None of the selected ideas are on {}.", line.name),
        (false, 1) => format!("Already on {}.", line.name),
        (false, _) => format!("All selected ideas are already on {}.", line.name),
    }
}
