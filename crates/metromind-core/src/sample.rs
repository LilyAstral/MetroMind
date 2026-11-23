//! A hand-made example brainstorm: 3 lines, 20 stations, 2 interchanges and a
//! few free-floating ideas. Used by tests and the app's "Load sample map".

use crate::model::{Line, LineId, MetroMap, Station, StationId};
use chrono::{DateTime, Duration, Utc};
use glam::Vec2;

/// `(title, x, y, energy, hours since last touch)`; positions are on the 40-unit grid.
type StationSpec = (&'static str, f32, f32, f64, i64);

const STATIONS: [StationSpec; 20] = [
    // 1–7: "Product"
    ("Offline mode", -400.0, -160.0, 0.35, 96),
    ("Quick capture", -240.0, -160.0, 0.9, 1),
    ("Keyboard-first", -80.0, -80.0, 0.5, 30),
    ("Shared maps", 80.0, 0.0, 0.95, 0),
    ("Templates", 240.0, 0.0, 0.3, 120),
    ("Public roadmap", 400.0, 80.0, 0.7, 6),
    ("Plugin API", 560.0, 160.0, 0.2, 200),
    // 8–12: "Community"
    ("Discord server", 80.0, -320.0, 0.4, 48),
    ("Weekly demos", 80.0, -160.0, 0.65, 12),
    ("Ambassadors", 80.0, 160.0, 0.25, 150),
    ("Meetup kit", 160.0, 320.0, 0.15, 300),
    ("Open office hours", 320.0, 320.0, 0.5, 20),
    // 13–17: "Funding"
    ("Pay what you want", 240.0, -240.0, 0.3, 72),
    ("Sponsorships", 400.0, -120.0, 0.45, 40),
    ("Team plan", 400.0, 240.0, 0.8, 3),
    ("Grant applications", 480.0, 400.0, 0.1, 400),
    ("Education discount", 640.0, 400.0, 0.35, 60),
    // 18–20: free-floating ideas
    ("Dark sky theme", -320.0, 160.0, 0.6, 8),
    ("Voice notes?", 640.0, -200.0, 0.2, 250),
    ("Print a poster map", -80.0, 400.0, 0.4, 50),
];

/// `(name, color, station numbers)`. "Shared maps" (4) and "Public roadmap" (6)
/// are the interchanges.
const LINES: [(&str, u8, &[u64]); 3] = [
    ("Product", 0, &[1, 2, 3, 4, 5, 6, 7]),
    ("Community", 1, &[8, 9, 4, 10, 11, 12]),
    ("Funding", 2, &[13, 14, 6, 15, 16, 17]),
];

pub fn sample_map(now: DateTime<Utc>) -> MetroMap {
    let mut map = MetroMap::default();
    for (i, &(title, x, y, energy, hours)) in STATIONS.iter().enumerate() {
        let id = StationId(i as u64 + 1);
        let touched = now - Duration::hours(hours);
        let station = Station {
            id,
            title: title.into(),
            notes: String::new(),
            position: Vec2::new(x, y),
            energy_at_touch: energy,
            last_touched: touched,
            created: touched - Duration::days(1),
        };
        map.stations.insert(id, station);
    }
    for (i, &(name, color_index, stations)) in LINES.iter().enumerate() {
        let id = LineId(STATIONS.len() as u64 + i as u64 + 1);
        let stations = stations.iter().map(|&n| StationId(n)).collect();
        let line = Line {
            id,
            name: name.into(),
            color_index,
            stations,
        };
        map.lines.insert(id, line);
    }
    map.next_id = (STATIONS.len() + LINES.len()) as u64 + 1;
    map
}
