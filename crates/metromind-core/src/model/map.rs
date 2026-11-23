use super::{Line, LineId, Station, StationId};
use crate::MapError;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Current file format version. Bump it when the JSON shape changes.
pub const SCHEMA_VERSION: u32 = 1;

/// The whole brainstorm. `BTreeMap`s keep iteration (and saved files) in a
/// deterministic order, so files diff cleanly and tests are stable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetroMap {
    pub schema_version: u32,
    /// Simple ID allocator shared by stations and lines. IDs are never reused.
    pub next_id: u64,
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

    /// The line bound to number key `index + 1`. Lines keep creation order
    /// because IDs only grow.
    pub fn line_at(&self, index: usize) -> Option<&Line> {
        self.lines.values().nth(index)
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
        Ok(())
    }
}

/// Order a station pair so (a, b) and (b, a) map to the same key.
pub fn canonical_pair(a: StationId, b: StationId) -> (StationId, StationId) {
    if a <= b { (a, b) } else { (b, a) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::sample_map;
    use chrono::DateTime;

    fn sample() -> MetroMap {
        sample_map(DateTime::UNIX_EPOCH)
    }

    #[test]
    fn sample_has_two_interchanges() {
        let map = sample();
        let interchanges = map
            .stations
            .keys()
            .filter(|&&id| map.is_interchange(id))
            .count();
        assert_eq!(interchanges, 2);
    }

    #[test]
    fn sample_is_valid() {
        assert!(sample().validate().is_ok());
    }

    #[test]
    fn segments_count_matches_lines() {
        let map = sample();
        let expected: usize = map.lines.values().map(|l| l.stations.len() - 1).sum();
        assert_eq!(map.segments().count(), expected);
    }

    #[test]
    fn bounds_of_empty_map_is_none() {
        assert!(MetroMap::default().bounds().is_none());
    }

    #[test]
    fn validate_catches_dangling_station() {
        let mut map = sample();
        if let Some(line) = map.lines.values_mut().next() {
            line.stations.push(StationId(9_999));
        }
        assert!(matches!(map.validate(), Err(MapError::UnknownStation(_))));
    }

    #[test]
    fn canonical_pair_is_symmetric() {
        let (a, b) = (StationId(3), StationId(7));
        assert_eq!(canonical_pair(a, b), canonical_pair(b, a));
    }
}
