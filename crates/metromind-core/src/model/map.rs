use super::{LINE_COLOR_COUNT, Line, LineId, Station, StationId};
use crate::MapError;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Current file format version. Bump it when the JSON shape changes.
pub const SCHEMA_VERSION: u32 = 3;

/// The whole brainstorm. `BTreeMap`s keep iteration (and saved files) in a
/// deterministic order, so files diff cleanly and tests are stable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetroMap {
    pub schema_version: u32,
    /// Simple ID allocator shared by stations and lines. IDs are never reused.
    pub next_id: u64,
    /// Index into the app's palette list. Saved so a map keeps its look.
    #[serde(default)]
    pub palette: u8,
    /// Lines in the order the number keys and side panel show them. A
    /// permutation of `lines`' keys.
    #[serde(default)]
    pub line_order: Vec<LineId>,
    pub stations: BTreeMap<StationId, Station>,
    pub lines: BTreeMap<LineId, Line>,
}

/// The connection between two consecutive stations on a line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub line: LineId,
    pub color_index: u8,
    /// Index of `from` within the line; a station inserted here goes at `index + 1`.
    pub index: usize,
    pub from: StationId,
    pub to: StationId,
}

impl Default for MetroMap {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            next_id: 1,
            palette: 0,
            line_order: Vec::new(),
            stations: BTreeMap::new(),
            lines: BTreeMap::new(),
        }
    }
}

impl MetroMap {
    pub fn is_empty(&self) -> bool {
        self.stations.is_empty() && self.lines.is_empty()
    }

    pub fn station(&self, id: StationId) -> Result<&Station, MapError> {
        self.stations.get(&id).ok_or(MapError::UnknownStation(id))
    }

    pub fn station_mut(&mut self, id: StationId) -> Result<&mut Station, MapError> {
        self.stations
            .get_mut(&id)
            .ok_or(MapError::UnknownStation(id))
    }

    pub fn line(&self, id: LineId) -> Result<&Line, MapError> {
        self.lines.get(&id).ok_or(MapError::UnknownLine(id))
    }

    pub fn line_mut(&mut self, id: LineId) -> Result<&mut Line, MapError> {
        self.lines.get_mut(&id).ok_or(MapError::UnknownLine(id))
    }

    /// Lines that pass through `id`. A station on two or more is an interchange.
    pub fn lines_through(&self, id: StationId) -> impl Iterator<Item = &Line> {
        self.lines.values().filter(move |line| line.contains(id))
    }

    pub fn is_interchange(&self, id: StationId) -> bool {
        self.lines_through(id).count() >= 2
    }

    /// The line bound to number key `index + 1`, in [`Self::line_order`].
    pub fn line_at(&self, index: usize) -> Option<&Line> {
        self.line_order.get(index).and_then(|id| self.lines.get(id))
    }

    /// Every line, in the user-chosen order.
    pub fn ordered_lines(&self) -> impl Iterator<Item = &Line> {
        self.line_order.iter().filter_map(|id| self.lines.get(id))
    }

    /// The lowest colour index no line uses yet, or `None` when all are taken.
    pub fn free_color_index(&self) -> Option<u8> {
        (0..LINE_COLOR_COUNT).find(|&c| self.lines.values().all(|l| l.color_index != c))
    }

    /// Bring a map read from an older schema up to the current one. Old files
    /// have no `line_order`, so it starts as ascending IDs (the old behaviour).
    /// v3 added station lists; their serde defaults (empty, bullets) cover v1/v2.
    pub(crate) fn upgrade(&mut self) {
        if self.schema_version < 2 {
            self.line_order = self.lines.keys().copied().collect();
        }
        self.schema_version = SCHEMA_VERSION;
    }

    /// The next ID the allocator would hand out, without consuming it.
    pub fn peek_id(&self) -> u64 {
        self.next_id
    }

    /// Make sure `id` will never be handed out again.
    pub(crate) fn reserve_id(&mut self, id: u64) {
        self.next_id = self.next_id.max(id + 1);
    }

    pub(crate) fn id_in_use(&self, id: u64) -> bool {
        self.stations.contains_key(&StationId(id)) || self.lines.contains_key(&LineId(id))
    }

    /// Every segment of every line, in line order.
    pub fn segments(&self) -> impl Iterator<Item = Segment> + '_ {
        self.lines.values().flat_map(|line| {
            line.stations
                .windows(2)
                .enumerate()
                .map(move |(index, pair)| Segment {
                    line: line.id,
                    color_index: line.color_index,
                    index,
                    from: pair[0],
                    to: pair[1],
                })
        })
    }

    /// Which lines run along each station pair, keyed by the pair in canonical
    /// (low, high) order. Used to draw shared segments as parallel strokes.
    pub fn shared_segments(&self) -> BTreeMap<(StationId, StationId), Vec<LineId>> {
        let mut shared: BTreeMap<_, Vec<LineId>> = BTreeMap::new();
        for segment in self.segments() {
            let key = canonical_pair(segment.from, segment.to);
            let lines = shared.entry(key).or_default();
            if !lines.contains(&segment.line) {
                lines.push(segment.line);
            }
        }
        shared
    }

    /// Axis-aligned bounds of all station positions, or `None` for an empty map.
    pub fn bounds(&self) -> Option<(Vec2, Vec2)> {
        let mut positions = self.stations.values().map(|s| s.position);
        let first = positions.next()?;
        Some(positions.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p))))
    }

    /// Check internal consistency of a map read from disk.
    pub fn validate(&self) -> Result<(), MapError> {
        for (&key, station) in &self.stations {
            if key != station.id {
                return Err(MapError::UnknownStation(key));
            }
            if station.id.0 >= self.next_id {
                return Err(MapError::DuplicateId(station.id.0));
            }
        }
        for (&key, line) in &self.lines {
            if key != line.id || line.id.0 >= self.next_id {
                return Err(MapError::UnknownLine(key));
            }
            if let Some(&missing) = line
                .stations
                .iter()
                .find(|s| !self.stations.contains_key(s))
            {
                return Err(MapError::UnknownStation(missing));
            }
        }
        let mut seen = BTreeSet::new();
        let order_ok = self.line_order.len() == self.lines.len()
            && self
                .line_order
                .iter()
                .all(|id| self.lines.contains_key(id) && seen.insert(*id));
        if !order_ok {
            return Err(MapError::BadLineOrder);
        }
        Ok(())
    }
}

/// Order a station pair so (a, b) and (b, a) map to the same key.
pub fn canonical_pair(a: StationId, b: StationId) -> (StationId, StationId) {
    if a <= b { (a, b) } else { (b, a) }
}

#[cfg(test)]
mod tests;
