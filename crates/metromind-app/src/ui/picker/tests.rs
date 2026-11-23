//! Picker rules: which rows are offered, greyed, where the highlight starts and moves.

use super::*;
use metromind_core::chrono::DateTime;
use metromind_core::editing::Edit;
use metromind_core::model::Line;
use metromind_core::sample::sample_map;

fn sample() -> MetroMap {
    sample_map(DateTime::UNIX_EPOCH)
}

fn connect_to(origin: u64, other: u64) -> Picker {
    Picker {
        origin: StationId(origin),
        target: Target::Station(StationId(other)),
        anchor: Pos2::ZERO,
        highlight: 0,
    }
}

#[test]
fn connecting_two_stations_on_one_line_greys_that_line() {
    let map = sample();
    let picker = connect_to(1, 2);
    let rows = rows(&map, &picker);
    assert_eq!(rows.len(), 4, "three lines plus New line");
    assert!(!rows[0].is_valid(), "both on Product");
    assert_eq!(
        rows[1].refusal.as_deref(),
        Some("Neither idea is on Community yet")
    );
}

#[test]
fn highlight_starts_on_the_only_valid_line_through_the_origin() {
    let map = sample();
    // Station 13 is on Funding only; connecting it to Offline mode (on Product).
    let picker = connect_to(13, 1);
    let rows = rows(&map, &picker);
    let start = start_highlight(&map, &picker, &rows);
    assert_eq!(rows[start].name, "Funding");
}

#[test]
fn step_skips_greyed_rows_and_stops_at_the_ends() {
    let map = sample();
    let picker = connect_to(1, 2);
    let rows = rows(&map, &picker);
    let down = step(&rows, 0, 1);
    assert!(rows[down].is_valid());
    assert_eq!(step(&rows, 0, -5), 0);
    assert_eq!(step(&rows, rows.len() - 1, 5), rows.len() - 1);
}

#[test]
fn branching_is_refused_on_lines_without_the_origin() {
    let map = sample();
    let picker = Picker {
        origin: StationId(18),
        target: Target::Spot(Vec2::new(0.0, 0.0)),
        anchor: Pos2::ZERO,
        highlight: 0,
    };
    let rows = rows(&map, &picker);
    assert!(rows.iter().take(3).all(|r| !r.is_valid()));
    assert_eq!(
        rows[0].refusal.as_deref(),
        Some("This idea isn't on Product")
    );
    assert!(rows[3].is_valid());
}

#[test]
fn new_line_is_greyed_when_all_colours_are_used() {
    let mut map = sample();
    while let Some(color_index) = map.free_color_index() {
        let line = Line {
            id: LineId(map.peek_id()),
            name: "filler".into(),
            color_index,
            stations: vec![StationId(18)],
            custom_color: None,
        };
        let position = map.line_order.len();
        Edit::CreateLine { line, position }
            .apply(&mut map)
            .expect("filler line");
    }
    let rows = rows(&map, &connect_to(1, 2));
    assert!(!rows.last().expect("new row").is_valid());
}
