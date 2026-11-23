//! Edits that add or remove stations, and put them on lines.

use super::{Edit, Membership};
use crate::MapError;
use crate::model::{LineId, MetroMap, Station, StationId};

pub(super) fn add_station(
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

pub(super) fn delete_station(map: &mut MetroMap, id: StationId) -> Result<Edit, MapError> {
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

pub(super) fn add_to_line(
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

pub(super) fn remove_from_line(
    map: &mut MetroMap,
    line: LineId,
    index: usize,
) -> Result<Edit, MapError> {
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
