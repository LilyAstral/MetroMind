//! End-to-end editing: a realistic session of edits, undone and redone.

use metromind_core::Vec2;
use metromind_core::chrono::{DateTime, Duration};
use metromind_core::clock::{Clock, FakeClock};
use metromind_core::editing::{Edit, Membership, UndoStack, touch};
use metromind_core::energy::{EnergySettings, Touch};
use metromind_core::model::{Line, LineId, MetroMap, Station, StationId};

fn station(map: &MetroMap, title: &str, at: Vec2, clock: &FakeClock) -> Station {
    Station::new(
        StationId(map.peek_id()),
        title,
        at,
        clock.now(),
        &EnergySettings::default(),
    )
}

#[test]
fn a_session_of_edits_can_be_fully_undone_and_redone() {
    let clock = FakeClock::new(DateTime::UNIX_EPOCH);
    let settings = EnergySettings::default();
    let mut map = MetroMap::default();
    let mut stack = UndoStack::default();
    let mut apply = |map: &mut MetroMap, edit: Edit| {
        stack
            .apply(map, edit, &clock, &settings)
            .expect("edit applies");
    };

    let a = station(&map, "A", Vec2::ZERO, &clock);
    let a_id = a.id;
    apply(
        &mut map,
        Edit::AddStation {
            station: a,
            memberships: vec![],
        },
    );
    let b = station(&map, "B", Vec2::new(160.0, 40.0), &clock);
    let b_id = b.id;
    apply(
        &mut map,
        Edit::AddStation {
            station: b,
            memberships: vec![],
        },
    );
    let line = Line {
        id: LineId(map.peek_id()),
        name: "Theme".into(),
        color_index: 0,
        stations: vec![a_id],
    };
    let line_id = line.id;
    apply(&mut map, Edit::CreateLine { line });
    apply(
        &mut map,
        Edit::AddToLine {
            line: line_id,
            station: b_id,
            index: 1,
        },
    );
    let mid = station(&map, "Mid", Vec2::new(80.0, 0.0), &clock);
    let mid_id = mid.id;
    apply(
        &mut map,
        Edit::AddStation {
            station: mid,
            memberships: vec![Membership {
                line: line_id,
                index: 1,
            }],
        },
    );
    apply(
        &mut map,
        Edit::Rename {
            id: a_id,
            title: "Alpha".into(),
        },
    );
    apply(&mut map, Edit::DeleteStation { id: b_id });

    let finished = map.clone();
    assert_eq!(map.lines[&line_id].stations, vec![a_id, mid_id]);

    let mut stack_undo = 0;
    while stack.undo(&mut map).expect("undo") {
        stack_undo += 1;
    }
    assert_eq!(stack_undo, 7);
    assert!(map.stations.is_empty() && map.lines.is_empty());

    while stack.redo(&mut map).expect("redo") {}
    assert_eq!(map, finished);
}

#[test]
fn energy_decays_and_rekindle_warms_it_back_up() {
    let clock = FakeClock::new(DateTime::UNIX_EPOCH);
    let settings = EnergySettings::default();
    let mut map = MetroMap::default();
    let idea = station(&map, "Idea", Vec2::ZERO, &clock);
    let id = idea.id;
    map.stations.insert(id, idea);

    let energy = |map: &MetroMap| map.stations[&id].energy(clock.now(), &settings);
    assert!((energy(&map) - 0.6).abs() < 1e-9);

    clock.advance(Duration::days(6));
    assert!(
        (energy(&map) - 0.15).abs() < 1e-9,
        "two half-lives → quarter"
    );
    assert!(settings.visible(0.0) >= settings.visual_floor);

    touch(&mut map, id, Touch::Rekindle, &clock, &settings).expect("rekindle");
    assert!((energy(&map) - 0.45).abs() < 1e-9);
}
