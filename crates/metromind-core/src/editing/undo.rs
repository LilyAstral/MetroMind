use super::Edit;
use crate::MapError;
use crate::clock::Clock;
use crate::energy::{EnergySettings, Touch, bump};
use crate::model::{MetroMap, StationId};

/// Oldest edits are dropped beyond this, so memory stays bounded.
const MAX_HISTORY: usize = 1_000;

/// Two stacks of inverse edits. Applying a new edit clears the redo stack.
#[derive(Debug, Default)]
pub struct UndoStack {
    undo: Vec<Edit>,
    redo: Vec<Edit>,
}

impl UndoStack {
    /// Apply a user edit, remember its inverse, and bump the touched station's
    /// energy. This is the single place where edits add energy. Undo and redo
    /// don't bump: they replay history rather than record new attention.
    pub fn apply(
        &mut self,
        map: &mut MetroMap,
        edit: Edit,
        clock: &dyn Clock,
        settings: &EnergySettings,
    ) -> Result<(), MapError> {
        let touched = edit.touches();
        let inverse = edit.apply(map)?;
        push_bounded(&mut self.undo, inverse);
        self.redo.clear();
        for (id, kind) in touched {
            touch(map, id, kind, clock, settings)?;
        }
        Ok(())
    }

    /// Undo the most recent edit. Returns `false` when there is nothing to undo.
    pub fn undo(&mut self, map: &mut MetroMap) -> Result<bool, MapError> {
        replay(map, &mut self.undo, &mut self.redo)
    }

    /// Redo the most recently undone edit. Returns `false` when there is nothing to redo.
    pub fn redo(&mut self, map: &mut MetroMap) -> Result<bool, MapError> {
        replay(map, &mut self.redo, &mut self.undo)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Forget all history, e.g. after opening another file.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

fn replay(map: &mut MetroMap, from: &mut Vec<Edit>, to: &mut Vec<Edit>) -> Result<bool, MapError> {
    let Some(edit) = from.pop() else {
        return Ok(false);
    };
    push_bounded(to, edit.apply(map)?);
    Ok(true)
}

fn push_bounded(stack: &mut Vec<Edit>, edit: Edit) {
    if stack.len() >= MAX_HISTORY {
        stack.remove(0);
    }
    stack.push(edit);
}

/// Record attention that is not an undoable change (selecting, rekindling).
pub fn touch(
    map: &mut MetroMap,
    id: StationId,
    kind: Touch,
    clock: &dyn Clock,
    settings: &EnergySettings,
) -> Result<(), MapError> {
    bump(map.station_mut(id)?, kind, clock.now(), settings);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::FakeClock;
    use crate::sample::sample_map;
    use approx::assert_relative_eq;
    use chrono::{DateTime, Duration};

    fn setup() -> (MetroMap, FakeClock, EnergySettings) {
        let clock = FakeClock::new(DateTime::UNIX_EPOCH);
        (sample_map(clock.now()), clock, EnergySettings::default())
    }

    fn some_station(map: &MetroMap) -> StationId {
        *map.stations.keys().next().expect("sample has stations")
    }

    #[test]
    fn undo_then_redo_restores_states() {
        let (mut map, clock, settings) = setup();
        let mut stack = UndoStack::default();
        let id = some_station(&map);
        let edit = Edit::Rename {
            id,
            title: "new title".into(),
        };
        stack
            .apply(&mut map, edit, &clock, &settings)
            .expect("applies");
        assert_eq!(map.stations[&id].title, "new title");
        assert!(stack.undo(&mut map).expect("undo"));
        assert_ne!(map.stations[&id].title, "new title");
        assert!(stack.redo(&mut map).expect("redo"));
        assert_eq!(map.stations[&id].title, "new title");
    }

    #[test]
    fn new_edit_clears_redo() {
        let (mut map, clock, settings) = setup();
        let mut stack = UndoStack::default();
        let id = some_station(&map);
        let rename = |t: &str| Edit::Rename {
            id,
            title: t.into(),
        };
        stack
            .apply(&mut map, rename("a"), &clock, &settings)
            .expect("a");
        stack.undo(&mut map).expect("undo");
        stack
            .apply(&mut map, rename("b"), &clock, &settings)
            .expect("b");
        assert!(!stack.can_redo());
    }

    #[test]
    fn empty_stack_undo_is_noop() {
        let (mut map, ..) = setup();
        assert!(!UndoStack::default().undo(&mut map).expect("ok"));
    }

    #[test]
    fn edits_bump_energy_once() {
        let (mut map, clock, settings) = setup();
        let mut stack = UndoStack::default();
        let id = some_station(&map);
        map.station_mut(id).expect("exists").energy_at_touch = 0.4;
        map.station_mut(id).expect("exists").last_touched = clock.now();
        clock.advance(Duration::days(3));
        let edit = Edit::SetNotes {
            id,
            notes: "n".into(),
        };
        stack
            .apply(&mut map, edit, &clock, &settings)
            .expect("applies");
        assert_relative_eq!(map.stations[&id].energy_at_touch, 0.2 + 0.2, epsilon = 1e-9);
        stack.undo(&mut map).expect("undo");
        assert_relative_eq!(map.stations[&id].energy_at_touch, 0.4, epsilon = 1e-9);
    }

    #[test]
    fn history_is_bounded() {
        let mut stack = Vec::new();
        for i in 0..(MAX_HISTORY + 5) {
            push_bounded(
                &mut stack,
                Edit::DeleteLine {
                    id: crate::model::LineId(i as u64),
                },
            );
        }
        assert_eq!(stack.len(), MAX_HISTORY);
    }
}
