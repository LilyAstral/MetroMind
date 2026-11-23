//! Ghost markers: where each dragged station will land when released.

use super::PlanInput;
use metromind_core::Vec2;
use metromind_core::geometry::{FisheyeLens, GRID_CELL, snap_to_grid};

/// One ghost per dragged station, at its grid-snapped landing spot, bent by the lens.
pub(super) fn build_ghosts(input: &PlanInput, lens: FisheyeLens) -> Vec<Vec2> {
    let Some(drag) = input.drag else {
        return Vec::new();
    };
    drag.moving
        .iter()
        .map(|&(_, start)| {
            let landing = snap_to_grid(start + drag.delta, GRID_CELL);
            lens.apply(input.camera.world_to_view(landing))
        })
        .collect()
}
