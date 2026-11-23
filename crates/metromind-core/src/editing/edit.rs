use super::line_ops::{create_line, delete_line, move_line, set_line_color, set_palette};
use super::station_ops::{add_station, add_to_line, delete_station, remove_from_line};
use crate::MapError;
use crate::energy::Touch;
use crate::model::{Line, LineId, ListStyle, MetroMap, NoteItem, Station, StationId};
use glam::Vec2;

#[cfg(test)]
mod tests;

/// A station's place on one line, recorded so deletion can be undone exactly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Membership {
    pub line: LineId,
    pub index: usize,
}

/// One undoable change to the map. The `match` in [`Edit::apply`] is
/// exhaustive, so a new variant has to be handled everywhere.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    /// Insert a station, optionally into lines at given positions.
    AddStation {
        station: Station,
        memberships: Vec<Membership>,
    },
    /// Remove a station and take it off every line it was on.
    DeleteStation {
        id: StationId,
    },
    MoveStation {
        id: StationId,
        to: Vec2,
    },
    Rename {
        id: StationId,
        title: String,
    },
    SetNotes {
        id: StationId,
        notes: String,
    },
    /// Replace a station's whole list, as `SetNotes` replaces its text.
    SetNoteItems {
        id: StationId,
        items: Vec<NoteItem>,
    },
    SetListStyle {
        id: StationId,
        style: ListStyle,
    },
    AddToLine {
        line: LineId,
        station: StationId,
        index: usize,
    },
    RemoveFromLine {
        line: LineId,
        index: usize,
    },
    /// Insert a line at `position` in the line order.
    CreateLine {
        line: Line,
        position: usize,
    },
    DeleteLine {
        id: LineId,
    },
    RenameLine {
        id: LineId,
        name: String,
    },
    /// Move a line to `to` in the line order.
    MoveLine {
        id: LineId,
        to: usize,
    },
    SetPalette {
        palette: u8,
    },
    /// `None` resets the line to its palette colour.
    SetLineColor {
        id: LineId,
        custom: Option<[u8; 3]>,
    },
    /// Several edits undone as one step. Applied in order; if one fails, the
    /// ones already applied are rolled back.
    Batch(Vec<Edit>),
}

impl Edit {
    /// Make the change and return the edit that undoes it. On error the map is
    /// left untouched.
    pub fn apply(self, map: &mut MetroMap) -> Result<Edit, MapError> {
        match self {
            Edit::AddStation {
                station,
                memberships,
            } => add_station(map, station, &memberships),
            Edit::DeleteStation { id } => delete_station(map, id),
            Edit::MoveStation { id, to } => {
                let old = std::mem::replace(&mut map.station_mut(id)?.position, to);
                Ok(Edit::MoveStation { id, to: old })
            }
            Edit::Rename { id, title } => {
                let old = std::mem::replace(&mut map.station_mut(id)?.title, title);
                Ok(Edit::Rename { id, title: old })
            }
            Edit::SetNotes { id, notes } => {
                let old = std::mem::replace(&mut map.station_mut(id)?.notes, notes);
                Ok(Edit::SetNotes { id, notes: old })
            }
            Edit::SetNoteItems { id, items } => {
                let old = std::mem::replace(&mut map.station_mut(id)?.items, items);
                Ok(Edit::SetNoteItems { id, items: old })
            }
            Edit::SetListStyle { id, style } => {
                let old = std::mem::replace(&mut map.station_mut(id)?.list_style, style);
                Ok(Edit::SetListStyle { id, style: old })
            }
            Edit::AddToLine {
                line,
                station,
                index,
            } => add_to_line(map, line, station, index),
            Edit::RemoveFromLine { line, index } => remove_from_line(map, line, index),
            Edit::CreateLine { line, position } => create_line(map, line, position),
            Edit::DeleteLine { id } => delete_line(map, id),
            Edit::RenameLine { id, name } => {
                let old = std::mem::replace(&mut map.line_mut(id)?.name, name);
                Ok(Edit::RenameLine { id, name: old })
            }
            Edit::MoveLine { id, to } => move_line(map, id, to),
            Edit::SetPalette { palette } => Ok(set_palette(map, palette)),
            Edit::SetLineColor { id, custom } => set_line_color(map, id, custom),
            Edit::Batch(edits) => apply_batch(map, edits),
        }
    }

    /// Which stations this edit counts as attention for, and how. Energy is
    /// bumped by [`super::UndoStack::apply`] only, never on undo/redo. A
    /// station appears at most once (the first touch wins).
    pub fn touches(&self) -> Vec<(StationId, Touch)> {
        let all: Vec<(StationId, Touch)> = match self {
            Edit::Batch(edits) => edits.iter().flat_map(Edit::touches).collect(),
            single => single.touch().into_iter().collect(),
        };
        let mut out: Vec<(StationId, Touch)> = Vec::new();
        for (id, kind) in all {
            if !out.iter().any(|(seen, _)| *seen == id) {
                out.push((id, kind));
            }
        }
        out
    }

    fn touch(&self) -> Option<(StationId, Touch)> {
        match self {
            Edit::Rename { id, .. }
            | Edit::SetNotes { id, .. }
            | Edit::SetNoteItems { id, .. }
            | Edit::SetListStyle { id, .. } => Some((*id, Touch::Edit)),
            Edit::MoveStation { id, .. } => Some((*id, Touch::Drag)),
            Edit::AddToLine { station, .. } => Some((*station, Touch::Connect)),
            Edit::CreateLine { line, .. } => line.stations.first().map(|&s| (s, Touch::Connect)),
            Edit::AddStation { .. }
            | Edit::DeleteStation { .. }
            | Edit::RemoveFromLine { .. }
            | Edit::DeleteLine { .. }
            | Edit::RenameLine { .. }
            | Edit::MoveLine { .. }
            | Edit::SetPalette { .. }
            | Edit::SetLineColor { .. }
            | Edit::Batch(_) => None,
        }
    }
}

/// Apply in order, collecting inverses. If one fails, undo the ones already
/// applied (newest first) so the map is exactly as it was.
fn apply_batch(map: &mut MetroMap, edits: Vec<Edit>) -> Result<Edit, MapError> {
    let mut inverses = Vec::with_capacity(edits.len());
    for edit in edits {
        match edit.apply(map) {
            Ok(inverse) => inverses.push(inverse),
            Err(err) => {
                for inverse in inverses.into_iter().rev() {
                    // Each inverse came from a successful apply, so it can only
                    // fail if the map changed underneath us.
                    let _ = inverse.apply(map);
                }
                return Err(err);
            }
        }
    }
    inverses.reverse();
    Ok(Edit::Batch(inverses))
}
