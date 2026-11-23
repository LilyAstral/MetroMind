//! Hit testing on the frame plan: which station or segment is under a point.

use super::to_pos2;
use super::{
    BASE_HIT_PX, FramePlan, MIN_HIT_SCALE, SEGMENT_HIT_PX, ScreenLine, ScreenStation, to_vec2,
};
use eframe::egui::{Pos2, Rect};
use metromind_core::geometry::distance_to_polyline;
use metromind_core::model::StationId;

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
