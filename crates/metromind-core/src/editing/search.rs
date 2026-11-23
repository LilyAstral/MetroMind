//! Finding stations by text. Used by the Ctrl+F search box.

use crate::model::{MetroMap, Station, StationId};

/// Up to `limit` stations matching `query`, best matches first. Matching is
/// case-insensitive. Ranks: title starts with the query, title contains it,
/// then notes contain it. Ties sort by title, then ID. An empty query finds
/// nothing.
pub fn search(map: &MetroMap, query: &str, limit: usize) -> Vec<StationId> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<(u8, String, StationId)> = map
        .stations
        .values()
        .filter_map(|station| {
            rank(station, &query).map(|r| (r, station.title.to_lowercase(), station.id))
        })
        .collect();
    hits.sort();
    hits.into_iter().take(limit).map(|(_, _, id)| id).collect()
}

fn rank(station: &Station, query: &str) -> Option<u8> {
    if station.title.to_lowercase().starts_with(query) {
        Some(0)
    } else if station.title.to_lowercase().contains(query) {
        Some(1)
    } else if station.notes.to_lowercase().contains(query) {
        Some(2)
    } else {
        None
    }
}
