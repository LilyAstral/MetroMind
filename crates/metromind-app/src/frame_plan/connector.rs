//! The connector preview: an octilinear line from the station a connection
//! drag started on to the pointer, or to the station the pointer is over.

use super::lines::{bounds, distort};
use super::{CORNER_RADIUS_PX, CORNER_SAMPLES, PlanInput, to_vec2, zoom_scale};
use metromind_core::Vec2;
use metromind_core::geometry::{FisheyeLens, round_corners, route};

pub(super) fn build_connector(input: &PlanInput, lens: FisheyeLens) -> Option<Vec<Vec2>> {
    let (from, pointer) = input.connect?;
    let origin = input.map.stations.get(&from)?;
    let target = match input.hovered.and_then(|id| input.map.stations.get(&id)) {
        Some(station) => station.position,
        None => input.camera.view_to_world(to_vec2(pointer)),
    };
    let start = input.camera.world_to_view(origin.position);
    let end = input.camera.world_to_view(target);
    let corner = CORNER_RADIUS_PX * zoom_scale(input.camera.zoom);
    let routed = round_corners(&route(start, end), corner, CORNER_SAMPLES);
    let (min, max) = bounds(&routed);
    Some(distort(&routed, lens, min, max))
}
