use metromind_core::MapError;
use metromind_core::chrono::{DateTime, Utc};
use metromind_core::model::{LineId, ListStyle, MetroMap, NoteItem, SCHEMA_VERSION, StationId};
use metromind_core::persistence::{from_json, load, save, temp_path, to_json};
use metromind_core::sample::sample_map;
use std::path::PathBuf;

/// A unique scratch folder per test so tests can run in parallel.
fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("metromind-test-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn now() -> DateTime<Utc> {
    "2026-11-23T12:00:00Z".parse().expect("valid timestamp")
}

#[test]
fn save_then_load_is_equal() {
    let dir = scratch_dir("round-trip");
    let path = dir.join("ideas.metromind");
    let map = sample_map(now());
    save(&map, &path).expect("save");
    assert_eq!(load(&path).expect("load"), map);
    assert!(!temp_path(&path).exists(), "temp file must be renamed away");
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn saving_over_an_existing_file_replaces_it() {
    let dir = scratch_dir("overwrite");
    let path = dir.join("ideas.metromind");
    save(&MetroMap::default(), &path).expect("first save");
    let map = sample_map(now());
    save(&map, &path).expect("second save");
    assert_eq!(load(&path).expect("load"), map);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn json_round_trip_is_stable() {
    let map = sample_map(now());
    let json = to_json(&map).expect("serialize");
    assert_eq!(
        to_json(&from_json(&json).expect("parse")).expect("re-serialize"),
        json
    );
}

#[test]
fn example_file_loads() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/example.metromind");
    let map = load(&path).expect("example loads");
    assert_eq!(map.stations.len(), 2);
    assert_eq!(map.stations[&StationId(1)].title, "Offline mode");
    assert_eq!(
        map.lines[&LineId(2)].stations,
        vec![StationId(1), StationId(4)]
    );
}

#[test]
fn newer_schema_is_rejected_with_friendly_message() {
    let json = to_json(&MetroMap {
        schema_version: 4,
        ..MetroMap::default()
    })
    .expect("serialize");
    let err = from_json(&json).expect_err("must reject");
    assert!(matches!(
        err,
        MapError::NewerSchema {
            found: 4,
            supported: 3
        }
    ));
    assert!(err.to_string().contains("newer MetroMind"));
}

#[test]
fn missing_file_is_an_io_error() {
    let err = load(&scratch_dir("missing").join("nope.metromind")).expect_err("must fail");
    assert!(matches!(err, MapError::Io(_)));
}

#[test]
fn inconsistent_file_is_rejected() {
    let json = r#"{ "schema_version": 1, "next_id": 3, "stations": {},
        "lines": { "1": { "id": 1, "name": "x", "color_index": 0, "stations": [2] } } }"#;
    assert!(matches!(from_json(json), Err(MapError::UnknownStation(_))));
}

#[test]
fn v1_fixture_loads_with_default_colours_and_order() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/example.metromind");
    let map = load(&path).expect("v1 file loads");
    assert_eq!(map.schema_version, SCHEMA_VERSION);
    assert_eq!(map.palette, 0);
    assert!(map.lines.values().all(|l| l.custom_color.is_none()));
    let ascending: Vec<LineId> = map.lines.keys().copied().collect();
    assert_eq!(map.line_order, ascending);
}

#[test]
fn v2_round_trip_keeps_order_palette_and_custom_colours() {
    let mut map = sample_map(now());
    map.palette = 4;
    let mut ids: Vec<LineId> = map.lines.keys().copied().collect();
    ids.reverse();
    map.line_order = ids;
    if let Some(line) = map.lines.values_mut().next() {
        line.custom_color = Some([10, 20, 30]);
    }
    let json = to_json(&map).expect("serialises");
    assert_eq!(from_json(&json).expect("parses"), map);
}

#[test]
fn v2_file_loads_with_empty_lists() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/v2.metromind");
    let map = load(&path).expect("v2 loads");
    assert_eq!(map.schema_version, SCHEMA_VERSION);
    let station = &map.stations[&StationId(1)];
    assert_eq!(station.notes, "Kept from v2");
    assert!(station.items.is_empty());
    assert_eq!(station.list_style, ListStyle::Bullets);
}

#[test]
fn v3_lists_survive_save_and_load() {
    let dir = scratch_dir("lists");
    let path = dir.join("ideas.metromind");
    let mut map = sample_map(now());
    let station = map.stations.get_mut(&StationId(1)).expect("station 1");
    station.items = vec![
        NoteItem {
            text: "Draft".into(),
            done: true,
        },
        NoteItem {
            text: "Test".into(),
            done: false,
        },
    ];
    station.list_style = ListStyle::Checkboxes;
    save(&map, &path).expect("save");
    assert_eq!(load(&path).expect("load"), map);
    std::fs::remove_dir_all(dir).ok();
}
