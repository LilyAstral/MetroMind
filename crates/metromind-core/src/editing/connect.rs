//! Rules for connecting stations to lines, and for cutting a connection.
//! Each function returns the [`Edit`] to apply, or a calm refusal.

use super::{Edit, Membership};
use crate::model::{Line, LineId, MetroMap, Station, StationId};

/// Why a connection can't be made. [`Self::reason`] gives the text shown to
/// the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectRefusal {
    NoSuchLine,
    NeitherOnLine,
    BothOnLine,
    NoFreeLine,
    NoSuchSegment,
    CannotSplit,
    OriginNotOnLine,
}

impl ConnectRefusal {
    pub fn reason(&self, line_name: &str) -> String {
        match self {
            Self::NoSuchLine => "That line doesn't exist".into(),
            Self::NeitherOnLine => format!("Neither idea is on {line_name} yet"),
            Self::BothOnLine => format!("Both ideas are already on {line_name}"),
            Self::NoFreeLine => "All 9 lines are in use".into(),
            Self::NoSuchSegment => "There's no connection here".into(),
            Self::CannotSplit => "All 9 lines are in use, so this line can't be split.".into(),
            Self::OriginNotOnLine => format!("This idea isn't on {line_name}"),
        }
    }
}

/// Connect `b` to `line` next to `a`, the station the drag started from.
pub fn connect(
    map: &MetroMap,
    line: LineId,
    a: StationId,
    b: StationId,
) -> Result<Edit, ConnectRefusal> {
    let line = map.lines.get(&line).ok_or(ConnectRefusal::NoSuchLine)?;
    let (anchor, new) = match (line.index_of(a), line.index_of(b)) {
        (Some(_), Some(_)) => return Err(ConnectRefusal::BothOnLine),
        (None, None) => return Err(ConnectRefusal::NeitherOnLine),
        (Some(_), None) => (a, b),
        (None, Some(_)) => (b, a),
    };
    let at = anchor_index(line, anchor);
    Ok(Edit::AddToLine {
        line: line.id,
        station: new,
        index: at,
    })
}

/// Where a new station goes so it sits beside `anchor` on its line.
fn anchor_index(line: &Line, anchor: StationId) -> usize {
    let index = line.index_of(anchor).unwrap_or(0);
    if index + 1 == line.stations.len() {
        line.stations.len()
    } else if index == 0 {
        0
    } else {
        index + 1
    }
}

/// A new two-station line from `a` to `b`, at the end of the line order.
pub fn connect_new_line(
    map: &MetroMap,
    a: StationId,
    b: StationId,
) -> Result<Edit, ConnectRefusal> {
    create_pair_line(map, map.peek_id(), vec![a, b])
}

/// Branch off `origin` onto `line`: `new` joins the line right beside
/// `origin`. Fails if `origin` isn't on the line.
pub fn branch(
    map: &MetroMap,
    line: LineId,
    origin: StationId,
    new: Station,
) -> Result<Edit, ConnectRefusal> {
    let source = map.lines.get(&line).ok_or(ConnectRefusal::NoSuchLine)?;
    if !source.contains(origin) {
        return Err(ConnectRefusal::OriginNotOnLine);
    }
    let index = anchor_index(source, origin);
    Ok(Edit::AddStation {
        station: new,
        memberships: vec![Membership { line, index }],
    })
}

/// Branch off `origin` onto a new two-station line. The new station is
/// added in the same batch, so one undo removes both.
pub fn branch_new_line(
    map: &MetroMap,
    origin: StationId,
    new: Station,
) -> Result<Edit, ConnectRefusal> {
    let new_id = new.id;
    // The new station takes its ID first, so the line gets the next one after it.
    let line_id = map.peek_id().max(new_id.0 + 1);
    let line = create_pair_line(map, line_id, vec![origin, new_id])?;
    Ok(Edit::Batch(vec![
        Edit::AddStation {
            station: new,
            memberships: Vec::new(),
        },
        line,
    ]))
}

/// A line with `stations`, the next free colour, the next name, at the end.
fn create_pair_line(
    map: &MetroMap,
    id: u64,
    stations: Vec<StationId>,
) -> Result<Edit, ConnectRefusal> {
    let color_index = map.free_color_index().ok_or(ConnectRefusal::NoFreeLine)?;
    let line = Line {
        id: LineId(id),
        name: format!("Line {}", map.lines.len() + 1),
        color_index,
        stations,
        custom_color: None,
    };
    Ok(Edit::CreateLine {
        line,
        position: map.line_order.len(),
    })
}

/// Remove the segment between `stations[index]` and `stations[index + 1]`.
/// Stations are never deleted, only taken off the line. Cutting a middle
/// segment splits the line in two.
pub fn disconnect(map: &MetroMap, line: LineId, index: usize) -> Result<Edit, ConnectRefusal> {
    let source = map.lines.get(&line).ok_or(ConnectRefusal::NoSuchLine)?;
    let n = source.stations.len();
    if index + 1 >= n {
        return Err(ConnectRefusal::NoSuchSegment);
    }
    if n == 2 {
        return Ok(remove_at(line, 1));
    }
    if index == 0 {
        return Ok(remove_at(line, 0));
    }
    if index == n - 2 {
        return Ok(remove_at(line, n - 1));
    }
    split(map, source, index)
}

fn remove_at(line: LineId, index: usize) -> Edit {
    Edit::RemoveFromLine { line, index }
}

/// Cut a middle segment: the tail becomes a new line placed right after.
fn split(map: &MetroMap, source: &Line, index: usize) -> Result<Edit, ConnectRefusal> {
    let color_index = map.free_color_index().ok_or(ConnectRefusal::CannotSplit)?;
    let n = source.stations.len();
    let tail: Vec<StationId> = source.stations[index + 1..].to_vec();
    let mut edits: Vec<Edit> = (index + 1..n)
        .rev()
        .map(|i| remove_at(source.id, i))
        .collect();
    let position = map
        .line_order
        .iter()
        .position(|&l| l == source.id)
        .map_or(map.line_order.len(), |p| p + 1);
    let new_line = Line {
        id: LineId(map.peek_id()),
        name: format!("{} 2", source.name),
        color_index,
        stations: tail,
        custom_color: None,
    };
    edits.push(Edit::CreateLine {
        line: new_line,
        position,
    });
    Ok(Edit::Batch(edits))
}
