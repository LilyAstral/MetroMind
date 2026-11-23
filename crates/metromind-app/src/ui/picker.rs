//! The line picker's logic: which lines are offered, which can take the
//! connection, where the highlight starts and moves, and the edit a pick makes.
//! Drawing lives in `picker_view.rs`.

use crate::render::colors::line_color;
use eframe::egui::{Color32, Pos2};
use metromind_core::Vec2;
use metromind_core::editing::{
    ConnectRefusal, Edit, branch, branch_new_line, connect, connect_new_line,
};
use metromind_core::model::{Line, LineId, MetroMap, Station, StationId};

/// What a connection drag ended on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    /// Connect to this station.
    Station(StationId),
    /// Branch to an empty, grid-snapped spot in the world.
    Spot(Vec2),
}

/// An open picker: opened where a connection drag was released.
#[derive(Debug, Clone, PartialEq)]
pub struct Picker {
    /// The station the drag started from.
    pub origin: StationId,
    pub target: Target,
    /// Screen position of the release point.
    pub anchor: Pos2,
    /// Index into [`rows`] of the highlighted row.
    pub highlight: usize,
}

/// One row of the picker: a line, or "New line".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Choice {
    Line(LineId),
    New,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub choice: Choice,
    pub name: String,
    /// Colour of the swatch; `None` for "New line".
    pub color: Option<Color32>,
    /// Why this row is greyed out, if it is.
    pub refusal: Option<String>,
}

impl Row {
    pub fn is_valid(&self) -> bool {
        self.refusal.is_none()
    }
}

/// Every line in line order, then "New line".
pub fn rows(map: &MetroMap, picker: &Picker) -> Vec<Row> {
    let mut rows: Vec<Row> = map
        .ordered_lines()
        .map(|line| Row {
            choice: Choice::Line(line.id),
            name: line.name.clone(),
            color: Some(line_color(map, line)),
            refusal: line_refusal(map, picker, line),
        })
        .collect();
    rows.push(Row {
        choice: Choice::New,
        name: "New line".into(),
        color: None,
        refusal: new_line_refusal(map, picker),
    });
    rows
}

fn line_refusal(map: &MetroMap, picker: &Picker, line: &Line) -> Option<String> {
    let result = match picker.target {
        Target::Station(other) => connect(map, line.id, picker.origin, other).map(|_| ()),
        Target::Spot(_) if line.contains(picker.origin) => Ok(()),
        Target::Spot(_) => Err(ConnectRefusal::OriginNotOnLine),
    };
    result.err().map(|refusal| refusal.reason(&line.name))
}

fn new_line_refusal(map: &MetroMap, picker: &Picker) -> Option<String> {
    if map.free_color_index().is_none() {
        return Some(ConnectRefusal::NoFreeLine.reason(""));
    }
    match picker.target {
        Target::Station(other) => connect_new_line(map, picker.origin, other).err(),
        Target::Spot(_) => None,
    }
    .map(|refusal| refusal.reason(""))
}

/// Exactly one valid line through the origin: that line. Otherwise the first
/// valid row.
pub fn start_highlight(map: &MetroMap, picker: &Picker, rows: &[Row]) -> usize {
    let through_origin: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| match row.choice {
            Choice::Line(id) => row.is_valid() && map.lines[&id].contains(picker.origin),
            Choice::New => false,
        })
        .map(|(i, _)| i)
        .collect();
    match through_origin[..] {
        [only] => only,
        _ => rows.iter().position(Row::is_valid).unwrap_or(0),
    }
}

/// Move the highlight `delta` valid rows (negative is up), stopping at the ends.
pub fn step(rows: &[Row], from: usize, delta: i32) -> usize {
    let mut at = from.min(rows.len().saturating_sub(1));
    for _ in 0..delta.unsigned_abs() {
        let next = if delta > 0 {
            (at + 1..rows.len()).find(|&i| rows[i].is_valid())
        } else {
            (0..at).rev().find(|&i| rows[i].is_valid())
        };
        match next {
            Some(i) => at = i,
            None => break,
        }
    }
    at
}

/// The edit a pick makes. `new` is the station a branch creates (its ID must
/// be the map's next free ID); it's ignored when connecting.
pub fn edit_for(
    map: &MetroMap,
    picker: &Picker,
    choice: Choice,
    new: Option<Station>,
) -> Result<Edit, ConnectRefusal> {
    match (picker.target, choice, new) {
        (Target::Station(other), Choice::Line(line), _) => connect(map, line, picker.origin, other),
        (Target::Station(other), Choice::New, _) => connect_new_line(map, picker.origin, other),
        (Target::Spot(_), Choice::Line(line), Some(station)) => {
            branch(map, line, picker.origin, station)
        }
        (Target::Spot(_), Choice::New, Some(station)) => {
            branch_new_line(map, picker.origin, station)
        }
        (Target::Spot(_), _, None) => Err(ConnectRefusal::NoSuchSegment),
    }
}

#[cfg(test)]
mod tests;
