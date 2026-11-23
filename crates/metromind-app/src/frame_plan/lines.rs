//! Line segments: routed octilinearly, rounded, laned, then bent by the lens.

use super::{
    CORNER_RADIUS_PX, CORNER_SAMPLES, PlanInput, SUBDIVIDE_STEP_PX, ScreenLine, to_pos2,
    world_position, zoom_scale,
};
use crate::render::colors::line_color;
use eframe::egui::Rect;
use metromind_core::Vec2;
use metromind_core::geometry::{FisheyeLens, offset_polyline, round_corners, route, subdivide};
use metromind_core::model::{LineId, canonical_pair};

pub(super) fn build_lines(
    input: &PlanInput,
    lens: FisheyeLens,
    line_width: f32,
) -> Vec<ScreenLine> {
    let shared = input.map.shared_segments();
    let corner = CORNER_RADIUS_PX * zoom_scale(input.camera.zoom);
    let visible = input.viewport.expand(line_width * 4.0);
    input
        .map
        .segments()
        .filter_map(|segment| {
            let line = input.map.lines.get(&segment.line)?;
            let (lo, hi) = canonical_pair(segment.from, segment.to);
            let view = |id| {
                let station = input.map.stations.get(&id)?;
                Some(input.camera.world_to_view(world_position(input, station)))
            };
            let routed = round_corners(&route(view(lo)?, view(hi)?), corner, CORNER_SAMPLES);
            let lanes = shared.get(&(lo, hi)).map_or(&[][..], Vec::as_slice);
            let points = offset_polyline(&routed, lane_offset(lanes, segment.line, line_width));
            let (min, max) = bounds(&points);
            if !visible.intersects(Rect::from_min_max(to_pos2(min), to_pos2(max))) {
                return None;
            }
            Some(ScreenLine {
                line: segment.line,
                color: line_color(input.map, line),
                dimmed: input.highlight.is_some_and(|h| h != segment.line),
                insert_at: segment.index + 1,
                points: distort(&points, lens, min, max),
            })
        })
        .collect()
}

/// Lines sharing a segment sit side by side, centered on the true route.
fn lane_offset(lanes: &[LineId], line: LineId, width: f32) -> f32 {
    let index = lanes.iter().position(|&l| l == line).unwrap_or(0) as f32;
    let center = (lanes.len().max(1) - 1) as f32 / 2.0;
    (index - center) * width
}

/// Subdivide only where the lens can actually bend the polyline.
pub(super) fn distort(points: &[Vec2], lens: FisheyeLens, min: Vec2, max: Vec2) -> Vec<Vec2> {
    if !lens.touches_box(min, max) {
        return points.to_vec();
    }
    subdivide(points, SUBDIVIDE_STEP_PX)
        .into_iter()
        .map(|p| lens.apply(p))
        .collect()
}

pub(super) fn bounds(points: &[Vec2]) -> (Vec2, Vec2) {
    let first = points.first().copied().unwrap_or(Vec2::ZERO);
    points
        .iter()
        .fold((first, first), |(lo, hi), &p| (lo.min(p), hi.max(p)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_segment_lanes_are_centered() {
        let (a, b) = (LineId(1), LineId(2));
        assert_eq!(lane_offset(&[a], a, 6.0), 0.0);
        assert_eq!(lane_offset(&[a, b], a, 6.0), -3.0);
        assert_eq!(lane_offset(&[a, b], b, 6.0), 3.0);
    }
}
