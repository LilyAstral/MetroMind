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
