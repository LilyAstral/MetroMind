use crate::MapError;
use crate::energy::Touch;
use crate::model::{Line, LineId, MetroMap, Station, StationId};
use glam::Vec2;

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
    AddToLine {
        line: LineId,
        station: StationId,
        index: usize,
    },
    RemoveFromLine {
        line: LineId,
        index: usize,
    },
    CreateLine {
        line: Line,
    },
    DeleteLine {
        id: LineId,
    },
    RenameLine {
        id: LineId,
        name: String,
    },
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
            Edit::AddToLine {
                line,
                station,
                index,
            } => add_to_line(map, line, station, index),
            Edit::RemoveFromLine { line, index } => remove_from_line(map, line, index),
            Edit::CreateLine { line } => create_line(map, line),
            Edit::DeleteLine { id } => {
                let line = map.lines.remove(&id).ok_or(MapError::UnknownLine(id))?;
                Ok(Edit::CreateLine { line })
            }
            Edit::RenameLine { id, name } => {
                let old = std::mem::replace(&mut map.line_mut(id)?.name, name);
                Ok(Edit::RenameLine { id, name: old })
            }
        }
    }

    /// Which station this edit counts as attention for, and how. Energy is
    /// bumped by [`super::UndoStack::apply`] only, never on undo/redo.
    pub fn touch(&self) -> Option<(StationId, Touch)> {
        match self {
            Edit::Rename { id, .. } | Edit::SetNotes { id, .. } => Some((*id, Touch::Edit)),
            Edit::MoveStation { id, .. } => Some((*id, Touch::Drag)),
            Edit::AddToLine { station, .. } => Some((*station, Touch::Connect)),
            Edit::CreateLine { line } => line.stations.first().map(|&s| (s, Touch::Connect)),
            Edit::AddStation { .. }
            | Edit::DeleteStation { .. }
            | Edit::RemoveFromLine { .. }
            | Edit::DeleteLine { .. }
            | Edit::RenameLine { .. } => None,
        }
    }
}

fn add_station(
    map: &mut MetroMap,
    station: Station,
    memberships: &[Membership],
) -> Result<Edit, MapError> {
    let id = station.id;
    if map.id_in_use(id.0) {
        return Err(MapError::DuplicateId(id.0));
    }
    for m in memberships {
        let len = map.line(m.line)?.stations.len();
        if m.index > len {
            return Err(MapError::BadIndex { index: m.index });
        }
    }
    for m in memberships {
        map.line_mut(m.line)?.stations.insert(m.index, id);
    }
    map.reserve_id(id.0);
    map.stations.insert(id, station);
    Ok(Edit::DeleteStation { id })
}

fn delete_station(map: &mut MetroMap, id: StationId) -> Result<Edit, MapError> {
    let station = map
        .stations
        .remove(&id)
        .ok_or(MapError::UnknownStation(id))?;
    let mut memberships = Vec::new();
    for line in map.lines.values_mut() {
        if let Some(index) = line.index_of(id) {
            line.stations.remove(index);
            memberships.push(Membership {
                line: line.id,
                index,
            });
        }
    }
    Ok(Edit::AddStation {
        station,
        memberships,
    })
}

fn add_to_line(
    map: &mut MetroMap,
    line: LineId,
    station: StationId,
    index: usize,
) -> Result<Edit, MapError> {
    map.station(station)?;
    let stations = &mut map.line_mut(line)?.stations;
    if stations.contains(&station) {
        return Err(MapError::AlreadyOnLine);
    }
    if index > stations.len() {
        return Err(MapError::BadIndex { index });
    }
    stations.insert(index, station);
    Ok(Edit::RemoveFromLine { line, index })
}

fn remove_from_line(map: &mut MetroMap, line: LineId, index: usize) -> Result<Edit, MapError> {
    let stations = &mut map.line_mut(line)?.stations;
    if index >= stations.len() {
        return Err(MapError::BadIndex { index });
    }
    let station = stations.remove(index);
    Ok(Edit::AddToLine {
        line,
        station,
        index,
    })
}

fn create_line(map: &mut MetroMap, line: Line) -> Result<Edit, MapError> {
    let id = line.id;
    if map.id_in_use(id.0) {
        return Err(MapError::DuplicateId(id.0));
    }
    if let Some(&missing) = line.stations.iter().find(|s| !map.stations.contains_key(s)) {
        return Err(MapError::UnknownStation(missing));
    }
    map.reserve_id(id.0);
    map.lines.insert(id, line);
    Ok(Edit::DeleteLine { id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::EnergySettings;
    use crate::sample::sample_map;
    use chrono::DateTime;

    fn sample() -> MetroMap {
        sample_map(DateTime::UNIX_EPOCH)
    }

    fn first_line(map: &MetroMap) -> Line {
        map.lines
            .values()
            .next()
            .cloned()
            .expect("sample has lines")
    }

    /// Applying an edit and then its inverse must restore the map exactly.
    fn assert_round_trip(edit: Edit) {
        let original = sample();
        let mut map = original.clone();
        let inverse = edit.apply(&mut map).expect("edit applies");
        assert_ne!(map, original, "edit should change something");
        let redo = inverse.apply(&mut map).expect("inverse applies");
        assert_eq!(strip_ids(&map), strip_ids(&original));
        redo.apply(&mut map).expect("redo applies");
    }

    /// `next_id` legitimately only grows, so compare everything else.
    fn strip_ids(map: &MetroMap) -> MetroMap {
        MetroMap {
            next_id: 0,
            ..map.clone()
        }
    }

    #[test]
    fn every_edit_round_trips() {
        let map = sample();
        let line = first_line(&map);
        let station = line.stations[1];
        let new_id = StationId(map.peek_id());
        let fresh = Station::new(
            new_id,
            "new",
            Vec2::ZERO,
            DateTime::UNIX_EPOCH,
            &EnergySettings::default(),
        );
        let free = *map
            .stations
            .keys()
            .find(|&&id| map.lines_through(id).count() == 0)
            .expect("sample has a free station");
        let edits = [
            Edit::AddStation {
                station: fresh,
                memberships: vec![Membership {
                    line: line.id,
                    index: 1,
                }],
            },
            Edit::DeleteStation { id: station },
            Edit::MoveStation {
                id: station,
                to: Vec2::new(999.0, 0.0),
            },
            Edit::Rename {
                id: station,
                title: "renamed".into(),
            },
            Edit::SetNotes {
                id: station,
                notes: "note".into(),
            },
            Edit::AddToLine {
                line: line.id,
                station: free,
                index: 0,
            },
            Edit::RemoveFromLine {
                line: line.id,
                index: 0,
            },
            Edit::DeleteLine { id: line.id },
            Edit::RenameLine {
                id: line.id,
                name: "other".into(),
            },
        ];
        for edit in edits {
            assert_round_trip(edit);
        }
    }

    #[test]
    fn delete_interchange_restores_all_memberships() {
        let original = sample();
        let mut map = original.clone();
        let hub = *map
            .stations
            .keys()
            .find(|&&id| map.is_interchange(id))
            .expect("sample has an interchange");
        let inverse = Edit::DeleteStation { id: hub }
            .apply(&mut map)
            .expect("deletes");
        assert!(map.lines.values().all(|l| !l.contains(hub)));
        inverse.apply(&mut map).expect("restores");
        assert_eq!(map, original);
    }

    #[test]
    fn adding_twice_to_a_line_is_rejected() {
        let mut map = sample();
        let line = first_line(&map);
        let edit = Edit::AddToLine {
            line: line.id,
            station: line.stations[0],
            index: 0,
        };
        assert!(matches!(edit.apply(&mut map), Err(MapError::AlreadyOnLine)));
    }

    #[test]
    fn failed_edit_leaves_map_untouched() {
        let original = sample();
        let mut map = original.clone();
        let edit = Edit::Rename {
            id: StationId(9_999),
            title: "x".into(),
        };
        assert!(edit.apply(&mut map).is_err());
        assert_eq!(map, original);
    }
}
