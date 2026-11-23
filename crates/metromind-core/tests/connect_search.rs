//! Connecting, branching, disconnecting and searching, through the public API.

use metromind_core::MapError;
use metromind_core::chrono::DateTime;
use metromind_core::editing::{
    ConnectRefusal, Edit, connect, connect_new_line, disconnect, search,
};
use metromind_core::model::{LineId, MetroMap, StationId};
use metromind_core::sample::sample_map;

fn sample() -> MetroMap {
    sample_map(DateTime::UNIX_EPOCH)
}

/// The "Product" line (1..=7) in the sample, and a station not on it (18).
fn product(map: &MetroMap) -> (LineId, Vec<StationId>) {
    let line = map.line_at(0).expect("sample has lines");
    (line.id, line.stations.clone())
}

fn apply(map: &mut MetroMap, edit: Edit) {
    edit.apply(map).expect("edit applies");
}

fn stations(map: &MetroMap, line: LineId) -> Vec<u64> {
    map.lines[&line].stations.iter().map(|s| s.0).collect()
}

#[test]
fn connecting_from_the_last_station_appends() {
    let mut map = sample();
    let (line, s) = product(&map);
    let free = StationId(18);
    let edit = connect(&map, line, *s.last().unwrap(), free).unwrap();
    apply(&mut map, edit);
    assert_eq!(map.lines[&line].stations.last(), Some(&free));
}

#[test]
fn connecting_from_the_first_station_prepends() {
    let mut map = sample();
    let (line, s) = product(&map);
    let free = StationId(18);
    let edit = connect(&map, line, s[0], free).unwrap();
    apply(&mut map, edit);
    assert_eq!(map.lines[&line].stations[0], free);
}

#[test]
fn connecting_from_the_middle_inserts_after_it() {
    let mut map = sample();
    let (line, s) = product(&map);
    let free = StationId(18);
    let edit = connect(&map, line, s[2], free).unwrap();
    apply(&mut map, edit);
    assert_eq!(map.lines[&line].stations[3], free);
    assert_eq!(map.lines[&line].stations[2], s[2]);
}

#[test]
fn swapped_drag_works_when_only_the_target_is_on_the_line() {
    let mut map = sample();
    let (line, s) = product(&map);
    let free = StationId(18);
    // Drag started on the free station and ended on the line's first station.
    let edit = connect(&map, line, free, s[0]).unwrap();
    apply(&mut map, edit);
    assert_eq!(map.lines[&line].stations[0], free);
}

#[test]
fn neither_on_line_is_refused() {
    let map = sample();
    let (line, _) = product(&map);
    let refusal = connect(&map, line, StationId(18), StationId(19));
    assert_eq!(refusal, Err(ConnectRefusal::NeitherOnLine));
    assert_eq!(
        ConnectRefusal::NeitherOnLine.reason("Product"),
        "Neither idea is on Product yet"
    );
}

#[test]
fn both_on_line_is_refused_even_when_adjacent() {
    let map = sample();
    let (line, s) = product(&map);
    assert_eq!(
        connect(&map, line, s[0], s[1]),
        Err(ConnectRefusal::BothOnLine)
    );
}

#[test]
fn connect_new_line_creates_a_pair_at_the_end() {
    let mut map = sample();
    let edit = connect_new_line(&map, StationId(18), StationId(19)).unwrap();
    apply(&mut map, edit);
    let last = *map.line_order.last().unwrap();
    let created = &map.lines[&last];
    assert_eq!(created.stations, vec![StationId(18), StationId(19)]);
    assert_eq!(created.color_index, 3);
    assert_eq!(created.name, "Line 4");
    assert!(map.validate().is_ok());
}

#[test]
fn connect_new_line_refuses_when_all_colours_are_used() {
    let mut map = sample();
    for i in 0..6u64 {
        let line = metromind_core::model::Line {
            id: LineId(map.peek_id()),
            name: format!("L{i}"),
            color_index: map.free_color_index().unwrap(),
            stations: vec![StationId(18)],
            custom_color: None,
        };
        let position = map.line_order.len();
        apply(&mut map, Edit::CreateLine { line, position });
    }
    assert_eq!(
        connect_new_line(&map, StationId(18), StationId(19)),
        Err(ConnectRefusal::NoFreeLine)
    );
}

#[test]
fn disconnect_on_a_two_station_line_drops_the_second() {
    let mut map = sample();
    let line = LineId(map.peek_id());
    let make = Edit::CreateLine {
        line: metromind_core::model::Line {
            id: line,
            name: "Two".into(),
            color_index: 7,
            stations: vec![StationId(1), StationId(2)],
            custom_color: None,
        },
        position: map.line_order.len(),
    };
    apply(&mut map, make);
    let edit = disconnect(&map, line, 0).unwrap();
    apply(&mut map, edit);
    assert_eq!(stations(&map, line), vec![1]);
}

#[test]
fn disconnect_at_the_ends_removes_the_end_station() {
    let mut map = sample();
    let (line, _) = product(&map);
    let edit = disconnect(&map, line, 0).unwrap();
    apply(&mut map, edit);
    assert_eq!(stations(&map, line), vec![2, 3, 4, 5, 6, 7]);
    let edit = disconnect(&map, line, 4).unwrap();
    apply(&mut map, edit);
    assert_eq!(stations(&map, line), vec![2, 3, 4, 5, 6]);
}

#[test]
fn disconnect_in_the_middle_splits_the_line_and_undo_restores_it() {
    let original = sample();
    let mut map = original.clone();
    let (line, _) = product(&map);
    let edit = disconnect(&map, line, 2).unwrap();
    let inverse = {
        let mut m = map.clone();
        let inverse = edit.clone().apply(&mut m).unwrap();
        assert_eq!(stations(&m, line), vec![1, 2, 3]);
        let tail = m.line_order.iter().position(|&l| l == line).unwrap() + 1;
        let new_line = m.line_order[tail];
        assert_eq!(stations(&m, new_line), vec![4, 5, 6, 7]);
        assert_eq!(m.lines[&new_line].name, "Product 2");
        assert!(m.validate().is_ok());
        inverse
    };
    apply(&mut map, edit);
    inverse.apply(&mut map).unwrap();
    // `next_id` only grows, so compare the content that undo must restore.
    assert_eq!(map.lines, original.lines);
    assert_eq!(map.line_order, original.line_order);
    assert_eq!(map.stations, original.stations);
}

#[test]
fn splitting_with_all_colours_used_is_refused() {
    let mut map = sample();
    for i in 0..6u64 {
        let line = metromind_core::model::Line {
            id: LineId(map.peek_id()),
            name: format!("L{i}"),
            color_index: map.free_color_index().unwrap(),
            stations: vec![StationId(18)],
            custom_color: None,
        };
        let position = map.line_order.len();
        apply(&mut map, Edit::CreateLine { line, position });
    }
    let (line, _) = product(&map);
    assert_eq!(disconnect(&map, line, 2), Err(ConnectRefusal::CannotSplit));
}

#[test]
fn disconnect_rejects_a_missing_segment() {
    let map = sample();
    let (line, s) = product(&map);
    assert_eq!(
        disconnect(&map, line, s.len() - 1),
        Err(ConnectRefusal::NoSuchSegment)
    );
    assert!(matches!(
        Edit::RemoveFromLine { line, index: 99 }.apply(&mut map.clone()),
        Err(MapError::BadIndex { .. })
    ));
}

fn retitle(map: &mut MetroMap, id: u64, title: &str, notes: &str) {
    let station = map.stations.get_mut(&StationId(id)).unwrap();
    station.title = title.into();
    station.notes = notes.into();
}

#[test]
fn search_ranks_prefix_then_contains_then_notes() {
    let mut map = sample();
    retitle(&mut map, 18, "Dark sky theme", "");
    retitle(&mut map, 19, "Sky walks", "");
    retitle(&mut map, 20, "Print a poster map", "a sky full of ideas");
    assert_eq!(
        search(&map, "  SKY ", 10),
        vec![StationId(19), StationId(18), StationId(20)]
    );
    assert_eq!(search(&map, "sky", 2), vec![StationId(19), StationId(18)]);
}

#[test]
fn search_finds_nothing_for_an_empty_query() {
    assert!(search(&sample(), "   ", 10).is_empty());
}
