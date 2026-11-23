//! Finding stations by text. Used by the Ctrl+F search box.

use crate::model::{MetroMap, Station, StationId};

/// Up to `limit` stations matching `query`, best matches first. Matching is
/// case-insensitive. Ranks: title starts with the query, title contains it,
/// then notes or a list item contain it. Ties sort by title, then ID. An empty
/// query finds nothing.
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
    } else if station.notes.to_lowercase().contains(query)
        || station
            .items
            .iter()
            .any(|item| item.text.to_lowercase().contains(query))
    {
        Some(2)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::EnergySettings;
    use crate::model::{NoteItem, Station};
    use chrono::DateTime;
    use glam::Vec2;

    #[test]
    fn list_items_are_searched_after_notes() {
        let settings = EnergySettings::default();
        let now = DateTime::UNIX_EPOCH;
        let mut map = MetroMap::default();
        let mut add = |id: u64, title: &str| {
            let station = Station::new(StationId(id), title, Vec2::ZERO, now, &settings);
            map.stations.insert(StationId(id), station);
        };
        add(1, "Alpha");
        add(2, "Rocket launch");
        add(3, "Beta");
        map.stations
            .get_mut(&StationId(1))
            .expect("1")
            .items
            .push(NoteItem {
                text: "pick rocket fuel".into(),
                done: false,
            });
        map.stations.get_mut(&StationId(3)).expect("3").notes = "rocket notes".into();
        assert_eq!(
            search(&map, "rocket", 10),
            vec![StationId(2), StationId(1), StationId(3)]
        );
    }
}
