use super::*;
use crate::energy::EnergySettings;
use crate::sample::sample_map;
use chrono::DateTime;

fn sample() -> MetroMap {
    sample_map(DateTime::UNIX_EPOCH)
}

fn first_line(map: &MetroMap) -> Line {
    map.ordered_lines()
        .next()
        .cloned()
        .expect("sample has lines")
}

/// A new line holding one station, ready to be created.
fn new_line(map: &MetroMap, station: StationId) -> Line {
    Line {
        id: LineId(map.peek_id()),
        name: "Extra".into(),
        color_index: map.free_color_index().unwrap_or(0),
        stations: vec![station],
        custom_color: None,
    }
}

/// Applying an edit and then its inverse must restore the map exactly.
fn assert_round_trip(edit: Edit) {
    let original = sample();
    let mut map = original.clone();
    let inverse = edit.apply(&mut map).expect("edit applies");
    assert_ne!(map, original, "edit should change something");
    let redo = inverse.apply(&mut map).expect("inverse applies");
    assert_eq!(strip_ids(&map), strip_ids(&original));
    redo.apply(&mut map).expect("redo applies");
}

/// `next_id` legitimately only grows, so compare everything else.
fn strip_ids(map: &MetroMap) -> MetroMap {
    MetroMap {
        next_id: 0,
        ..map.clone()
    }
}

#[test]
fn every_edit_round_trips() {
    let map = sample();
    let line = first_line(&map);
    let station = line.stations[1];
    let new_id = StationId(map.peek_id());
    let fresh = Station::new(
        new_id,
        "new",
        Vec2::ZERO,
        DateTime::UNIX_EPOCH,
        &EnergySettings::default(),
    );
    let free = *map
        .stations
        .keys()
        .find(|&&id| map.lines_through(id).count() == 0)
        .expect("sample has a free station");
    let edits = [
        Edit::AddStation {
            station: fresh,
            memberships: vec![Membership {
                line: line.id,
                index: 1,
            }],
        },
        Edit::DeleteStation { id: station },
        Edit::MoveStation {
            id: station,
            to: Vec2::new(999.0, 0.0),
        },
        Edit::Rename {
            id: station,
            title: "renamed".into(),
        },
        Edit::SetNotes {
            id: station,
            notes: "note".into(),
        },
        Edit::AddToLine {
            line: line.id,
            station: free,
            index: 0,
        },
        Edit::RemoveFromLine {
            line: line.id,
            index: 0,
        },
        Edit::DeleteLine { id: line.id },
        Edit::RenameLine {
            id: line.id,
            name: "other".into(),
        },
        Edit::MoveLine { id: line.id, to: 2 },
        Edit::SetPalette { palette: 3 },
        Edit::SetLineColor {
            id: line.id,
            custom: Some([1, 2, 3]),
        },
        Edit::CreateLine {
            line: new_line(&map, free),
            position: 1,
        },
    ];
    for edit in edits {
        assert_round_trip(edit);
    }
}

#[test]
fn batch_round_trips_as_one_step() {
    let map = sample();
    let line = first_line(&map);
    let batch = Edit::Batch(vec![
        Edit::Rename {
            id: line.stations[0],
            title: "a".into(),
        },
        Edit::MoveLine { id: line.id, to: 1 },
        Edit::SetPalette { palette: 2 },
    ]);
    assert_round_trip(batch);
}

#[test]
fn batch_rolls_back_when_a_step_fails() {
    let original = sample();
    let mut map = original.clone();
    let line = first_line(&map);
    let batch = Edit::Batch(vec![
        Edit::Rename {
            id: line.stations[0],
            title: "changed".into(),
        },
        Edit::SetPalette { palette: 4 },
        Edit::Rename {
            id: StationId(9_999),
            title: "fails".into(),
        },
    ]);
    assert!(batch.apply(&mut map).is_err());
    assert_eq!(map, original);
}

#[test]
fn batch_touches_each_station_once() {
    let map = sample();
    let station = first_line(&map).stations[0];
    let batch = Edit::Batch(vec![
        Edit::MoveStation {
            id: station,
            to: Vec2::ZERO,
        },
        Edit::Rename {
            id: station,
            title: "x".into(),
        },
    ]);
    let touches = batch.touches();
    assert_eq!(touches, vec![(station, Touch::Drag)]);
}

#[test]
fn delete_line_restores_its_position() {
    let original = sample();
    let mut map = original.clone();
    let middle = original.line_order[1];
    let inverse = Edit::DeleteLine { id: middle }
        .apply(&mut map)
        .expect("deletes");
    assert!(!map.line_order.contains(&middle));
    inverse.apply(&mut map).expect("restores");
    assert_eq!(map.line_order, original.line_order);
}

#[test]
fn move_line_reorders_number_keys() {
    let mut map = sample();
    let first = map.line_order[0];
    Edit::MoveLine { id: first, to: 2 }
        .apply(&mut map)
        .expect("moves");
    assert_eq!(map.line_at(2).map(|l| l.id), Some(first));
    assert!(map.validate().is_ok());
}

#[test]
fn move_line_rejects_out_of_range() {
    let mut map = sample();
    let first = map.line_order[0];
    let edit = Edit::MoveLine { id: first, to: 3 };
    assert!(matches!(
        edit.apply(&mut map),
        Err(MapError::BadIndex { index: 3 })
    ));
}

#[test]
fn set_palette_returns_old_palette_as_inverse() {
    let mut map = sample();
    let inverse = Edit::SetPalette { palette: 5 }
        .apply(&mut map)
        .expect("applies");
    assert_eq!(map.palette, 5);
    assert_eq!(inverse, Edit::SetPalette { palette: 0 });
}

#[test]
fn delete_interchange_restores_all_memberships() {
    let original = sample();
    let mut map = original.clone();
    let hub = *map
        .stations
        .keys()
        .find(|&&id| map.is_interchange(id))
        .expect("sample has an interchange");
    let inverse = Edit::DeleteStation { id: hub }
        .apply(&mut map)
        .expect("deletes");
    assert!(map.lines.values().all(|l| !l.contains(hub)));
    inverse.apply(&mut map).expect("restores");
    assert_eq!(map, original);
}

#[test]
fn adding_twice_to_a_line_is_rejected() {
    let mut map = sample();
    let line = first_line(&map);
    let edit = Edit::AddToLine {
        line: line.id,
        station: line.stations[0],
        index: 0,
    };
    assert!(matches!(edit.apply(&mut map), Err(MapError::AlreadyOnLine)));
}

#[test]
fn failed_edit_leaves_map_untouched() {
    let original = sample();
    let mut map = original.clone();
    let edit = Edit::Rename {
        id: StationId(9_999),
        title: "x".into(),
    };
    assert!(edit.apply(&mut map).is_err());
    assert_eq!(map, original);
}

#[test]
fn list_edits_round_trip() {
    let map = sample();
    let id = *map.stations.keys().next().expect("stations");
    assert_round_trip(Edit::SetNoteItems {
        id,
        items: vec![NoteItem {
            text: "Step".into(),
            done: true,
        }],
    });
    assert_round_trip(Edit::SetListStyle {
        id,
        style: ListStyle::Checkboxes,
    });
}

#[test]
fn deleting_a_station_and_undoing_restores_its_list() {
    let mut map = sample();
    let id = *map.stations.keys().next().expect("stations");
    let items = vec![NoteItem {
        text: "Keep me".into(),
        done: false,
    }];
    Edit::SetNoteItems {
        id,
        items: items.clone(),
    }
    .apply(&mut map)
    .expect("set items");
    let inverse = Edit::DeleteStation { id }.apply(&mut map).expect("delete");
    assert!(!map.stations.contains_key(&id));
    inverse.apply(&mut map).expect("undo delete");
    assert_eq!(map.stations[&id].items, items);
}
