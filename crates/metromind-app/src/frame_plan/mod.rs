//! map + camera + lens + clock → [`FramePlan`]: everything a frame draws, in
//! screen space, computed once. Layers and hit testing only read it.

mod connector;
mod ghosts;
mod grid;
mod lines;
mod stations;
#[cfg(test)]
mod tests;

use crate::input::StationDrag;
use eframe::egui::{Pos2, Rect, pos2};
use metromind_core::Vec2;
use metromind_core::chrono::{DateTime, Utc};
use metromind_core::energy::EnergySettings;
use metromind_core::geometry::{Camera, FisheyeLens, distance_to_polyline};
use metromind_core::model::{LineId, MetroMap, Station, StationId};
use std::collections::BTreeSet;

pub const STATION_RADIUS: f32 = 7.0;
/// Hit radius at magnification 1; 22 px radius = a 44 px target.
pub const BASE_HIT_PX: f32 = 22.0;
/// Compressed stations near the lens rim still get a usable target.
const MIN_HIT_SCALE: f32 = 0.6;
pub const LABEL_THRESHOLD: f32 = 1.6;
pub const BASE_FONT_PX: f32 = 14.0;
const MAX_LABEL_SCALE: f32 = 2.2;
/// Hot ideas are always readable.
const HOT_LABEL_ENERGY: f64 = 0.6;
const SUBDIVIDE_STEP_PX: f32 = 6.0;
pub const LINE_WIDTH: f32 = 6.0;
const CORNER_RADIUS_PX: f32 = 18.0;
const CORNER_SAMPLES: usize = 6;
const MIN_GRID_SPACING_PX: f32 = 18.0;
/// How far a double-click may miss a segment and still hit it.
pub const SEGMENT_HIT_PX: f32 = 10.0;

/// Everything [`build`] needs. Bundled so the call site stays readable.
pub struct PlanInput<'a> {
    pub map: &'a MetroMap,
    pub camera: Camera,
    pub lens: FisheyeLens,
    pub lens_enabled: bool,
    pub now: DateTime<Utc>,
    pub energy: &'a EnergySettings,
    pub viewport: Rect,
    pub selection: &'a BTreeSet<StationId>,
    pub primary: Option<StationId>,
    pub hovered: Option<StationId>,
    /// Stations being dragged, with the unsnapped pointer movement.
    pub drag: Option<&'a StationDrag>,
    pub marquee: Option<Rect>,
    /// The station a connection drag started from, and the pointer.
    pub connect: Option<(StationId, Pos2)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenStation {
    pub id: StationId,
    pub pos: Vec2,
    pub radius: f32,
    pub magnification: f32,
    /// Visual energy, already floored for rendering.
    pub energy: f32,
    pub is_interchange: bool,
    pub line_count: usize,
    /// Color of the first line through the station, if any.
    pub color_index: Option<u8>,
    /// In the selection (drawn with a ring).
    pub selected: bool,
    /// The primary station of the selection.
    pub primary: bool,
    pub hovered: bool,
}

/// One drawn segment of a line, already routed, offset, bent and distorted.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenLine {
    pub line: LineId,
    pub color_index: u8,
    /// Where a station inserted on this segment goes in the line.
    pub insert_at: usize,
    pub points: Vec<Vec2>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenLabel {
    pub anchor: Vec2,
    pub text: String,
    pub size: f32,
    pub placeholder: bool,
    pub emphasized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridDot {
    pub pos: Vec2,
    pub magnification: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FramePlan {
    pub camera: Camera,
    pub lens: FisheyeLens,
    pub lens_enabled: bool,
    pub viewport: Rect,
    pub line_width: f32,
    pub grid: Vec<GridDot>,
    pub stations: Vec<ScreenStation>,
    pub lines: Vec<ScreenLine>,
    pub labels: Vec<ScreenLabel>,
    /// Where stations being dragged will land (snapped, through the lens).
    pub ghosts: Vec<Vec2>,
    pub marquee: Option<Rect>,
    /// The dashed preview of a connection drag, in screen space.
    pub connector: Option<Vec<Vec2>>,
}

impl Default for FramePlan {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            lens: FisheyeLens::default(),
            lens_enabled: false,
            viewport: Rect::NOTHING,
            line_width: LINE_WIDTH,
            grid: Vec::new(),
            stations: Vec::new(),
            lines: Vec::new(),
            labels: Vec::new(),
            ghosts: Vec::new(),
            marquee: None,
            connector: None,
        }
    }
}

impl FramePlan {
    /// The station under `p`: nearest within its magnified hit radius.
    pub fn station_at(&self, p: Pos2) -> Option<StationId> {
        let p = to_vec2(p);
        self.stations
            .iter()
            .map(|s| (s, s.pos.distance(p)))
            .filter(|(s, d)| *d <= BASE_HIT_PX * s.magnification.max(MIN_HIT_SCALE))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(s, _)| s.id)
    }

    pub fn segment_at(&self, p: Pos2) -> Option<&ScreenLine> {
        let p = to_vec2(p);
        self.lines
            .iter()
            .map(|l| (l, distance_to_polyline(p, &l.points)))
            .filter(|(_, d)| *d <= SEGMENT_HIT_PX)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(l, _)| l)
    }

    /// Stations whose on-screen position is inside `rect`, for box select.
    pub fn stations_in(&self, rect: Rect) -> Vec<StationId> {
        self.stations
            .iter()
            .filter(|s| rect.contains(to_pos2(s.pos)))
            .map(|s| s.id)
            .collect()
    }

    pub fn station(&self, id: StationId) -> Option<&ScreenStation> {
        self.stations.iter().find(|s| s.id == id)
    }
}

/// Stations, lines and sizes shrink a little when zoomed out and grow a
/// little when zoomed in, but stay readable at both extremes.
fn zoom_scale(zoom: f32) -> f32 {
    zoom.clamp(0.6, 1.4)
}

pub fn build(input: &PlanInput) -> FramePlan {
    let lens = if input.lens_enabled {
        input.lens
    } else {
        input.lens.disabled()
    };
    let line_width = LINE_WIDTH * zoom_scale(input.camera.zoom);
    let stations = stations::build_stations(input, lens);
    let labels = stations::build_labels(input, &stations);
    FramePlan {
        camera: input.camera,
        lens,
        lens_enabled: input.lens_enabled,
        viewport: input.viewport,
        line_width,
        grid: grid::build_grid(input.camera, lens, input.viewport),
        lines: lines::build_lines(input, lens, line_width),
        stations,
        labels,
        ghosts: ghosts::build_ghosts(input, lens),
        marquee: input.marquee,
        connector: connector::build_connector(input, lens),
    }
}

/// Where a station is drawn in world space: where it is, or where the drag has moved it.
pub fn world_position(input: &PlanInput, station: &Station) -> Vec2 {
    match input.drag {
        Some(drag) if drag.moving.iter().any(|(id, _)| *id == station.id) => {
            station.position + drag.delta
        }
        _ => station.position,
    }
}

pub fn to_pos2(v: Vec2) -> Pos2 {
    pos2(v.x, v.y)
}

pub fn to_vec2(p: Pos2) -> Vec2 {
    Vec2::new(p.x, p.y)
}
