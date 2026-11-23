//! Edits that change lines (not their stations): create, delete, reorder,
//! palette and colour.

use super::Edit;
use crate::MapError;
use crate::model::{Line, LineId, MetroMap};

pub(super) fn create_line(
    map: &mut MetroMap,
    line: Line,
    position: usize,
) -> Result<Edit, MapError> {
    let id = line.id;
    if map.id_in_use(id.0) {
        return Err(MapError::DuplicateId(id.0));
    }
    if position > map.line_order.len() {
        return Err(MapError::BadIndex { index: position });
    }
    if let Some(&missing) = line.stations.iter().find(|s| !map.stations.contains_key(s)) {
        return Err(MapError::UnknownStation(missing));
    }
    map.reserve_id(id.0);
    map.line_order.insert(position, id);
    map.lines.insert(id, line);
    Ok(Edit::DeleteLine { id })
}

pub(super) fn delete_line(map: &mut MetroMap, id: LineId) -> Result<Edit, MapError> {
    let position = order_index(map, id)?;
    let line = map.lines.remove(&id).ok_or(MapError::UnknownLine(id))?;
    map.line_order.remove(position);
    Ok(Edit::CreateLine { line, position })
}

pub(super) fn move_line(map: &mut MetroMap, id: LineId, to: usize) -> Result<Edit, MapError> {
    let from = order_index(map, id)?;
    if to >= map.line_order.len() {
        return Err(MapError::BadIndex { index: to });
    }
    map.line_order.remove(from);
    map.line_order.insert(to, id);
    Ok(Edit::MoveLine { id, to: from })
}

pub(super) fn set_palette(map: &mut MetroMap, palette: u8) -> Edit {
    let old = std::mem::replace(&mut map.palette, palette);
    Edit::SetPalette { palette: old }
}

pub(super) fn set_line_color(
    map: &mut MetroMap,
    id: LineId,
    custom: Option<[u8; 3]>,
) -> Result<Edit, MapError> {
    let old = std::mem::replace(&mut map.line_mut(id)?.custom_color, custom);
    Ok(Edit::SetLineColor { id, custom: old })
}

fn order_index(map: &MetroMap, id: LineId) -> Result<usize, MapError> {
    map.line_order
        .iter()
        .position(|&l| l == id)
        .ok_or(MapError::UnknownLine(id))
}
