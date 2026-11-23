use metromind_core::MapError;
use metromind_core::chrono::{DateTime, Utc};
use metromind_core::model::{LineId, MetroMap, StationId};
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
        schema_version: 2,
        ..MetroMap::default()
    })
    .expect("serialize");
    let err = from_json(&json).expect_err("must reject");
    assert!(matches!(
        err,
        MapError::NewerSchema {
            found: 2,
            supported: 1
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
