//! Plan-level tests: hit testing, labels, lens bending, and a timing benchmark.

use super::*;
use metromind_core::sample::sample_map;

fn plan_for(map: &MetroMap, lens_enabled: bool, selected: Option<StationId>) -> FramePlan {
    let now = DateTime::UNIX_EPOCH;
    let viewport = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 800.0));
    let camera = Camera {
        pan: Vec2::new(600.0, 400.0),
        zoom: 1.0,
    };
    build(&PlanInput {
        map,
        camera,
        lens: FisheyeLens {
            focus: Vec2::new(600.0, 400.0),
            ..FisheyeLens::default()
        },
        lens_enabled,
        now,
        energy: &EnergySettings::default(),
        viewport,
        selected,
        hovered: None,
        drag: None,
    })
}

#[test]
fn sample_plan_has_every_station_and_segment() {
    let map = sample_map(DateTime::UNIX_EPOCH);
    let plan = plan_for(&map, false, None);
    assert_eq!(plan.stations.len(), map.stations.len());
    assert_eq!(plan.lines.len(), map.segments().count());
    assert_eq!(plan.stations.iter().filter(|s| s.is_interchange).count(), 2);
}

#[test]
fn clicking_a_station_hits_it() {
    let map = sample_map(DateTime::UNIX_EPOCH);
    for lens in [false, true] {
        let plan = plan_for(&map, lens, None);
        for s in &plan.stations {
            assert_eq!(plan.station_at(to_pos2(s.pos)), Some(s.id), "lens={lens}");
        }
    }
}

#[test]
fn selected_and_hot_stations_are_labeled() {
    let map = sample_map(DateTime::UNIX_EPOCH);
    let cold = *map
        .stations
        .values()
        .find(|s| s.energy_at_touch < 0.2)
        .map(|s| &s.id)
        .expect("sample has a cold station");
    let unselected = plan_for(&map, false, None);
    let selected = plan_for(&map, false, Some(cold));
    assert!(selected.labels.len() > unselected.labels.len());
    assert!(unselected.labels.iter().any(|l| l.text == "Shared maps"));
}

#[test]
fn lens_bends_lines_with_extra_points() {
    let map = sample_map(DateTime::UNIX_EPOCH);
    let flat: usize = plan_for(&map, false, None)
        .lines
        .iter()
        .map(|l| l.points.len())
        .sum();
    let bent: usize = plan_for(&map, true, None)
        .lines
        .iter()
        .map(|l| l.points.len())
        .sum();
    assert!(bent > flat);
}

/// 500 stations on 9 long lines. Run with
/// `cargo test --release -p metromind-app -- --ignored --nocapture`.
#[test]
#[ignore = "timing benchmark"]
fn bench_500_stations() {
    use metromind_core::model::{Line, Station};
    let now = DateTime::UNIX_EPOCH;
    let energy = EnergySettings::default();
    let mut map = MetroMap::default();
    for i in 0..500u64 {
        let position = Vec2::new((i % 25) as f32 * 80.0, (i / 25) as f32 * 80.0);
        let station = Station::new(
            StationId(i + 1),
            format!("Idea {i}"),
            position,
            now,
            &energy,
        );
        map.stations.insert(station.id, station);
    }
    for l in 0..9u64 {
        let stations = (0..55)
            .map(|k| StationId(1 + (l * 55 + k * 7) % 500))
            .collect();
        let id = LineId(1000 + l);
        let line = Line {
            id,
            name: format!("L{l}"),
            color_index: l as u8,
            stations,
            custom_color: None,
        };
        map.lines.insert(id, line);
        map.line_order.push(id);
    }
    map.next_id = 2000;
    let viewport = Rect::from_min_max(pos2(0.0, 0.0), pos2(1600.0, 1000.0));
    let camera = Camera::fit(
        Vec2::ZERO,
        Vec2::new(1920.0, 1520.0),
        Vec2::ZERO,
        Vec2::new(1600.0, 1000.0),
        40.0,
    );
    let start = std::time::Instant::now();
    let frames = 200;
    for f in 0..frames {
        let lens = FisheyeLens {
            focus: Vec2::new(400.0 + f as f32 * 4.0, 500.0),
            ..FisheyeLens::default()
        };
        let input = PlanInput {
            map: &map,
            camera,
            lens,
            lens_enabled: true,
            now,
            energy: &energy,
            viewport,
            selected: None,
            hovered: None,
            drag: None,
        };
        std::hint::black_box(build(&input));
    }
    let per_frame = start.elapsed() / frames;
    println!("frame_plan::build with 500 stations: {per_frame:?} per frame");
    assert!(per_frame.as_millis() < 8, "plan too slow: {per_frame:?}");
}
