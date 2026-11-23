//! Stations and their labels, with the lens applied.

use super::{
    BASE_FONT_PX, HOT_LABEL_ENERGY, LABEL_THRESHOLD, MAX_LABEL_SCALE, MIN_HIT_SCALE, PlanInput,
    STATION_RADIUS, ScreenLabel, ScreenStation, world_position, zoom_scale,
};
use crate::render::colors::line_color;
use eframe::egui::Color32;
use metromind_core::Vec2;
use metromind_core::geometry::FisheyeLens;
use metromind_core::model::{LineId, MetroMap, StationId};
use std::collections::BTreeMap;

pub(super) fn build_stations(input: &PlanInput, lens: FisheyeLens) -> Vec<ScreenStation> {
    let colors = first_line_colors(input.map);
    let scale = zoom_scale(input.camera.zoom);
    input
        .map
        .stations
        .values()
        .map(|station| {
            let view = input.camera.world_to_view(world_position(input, station));
            let magnification = lens.magnification(view);
            let energy = input
                .energy
                .visible(station.energy(input.now, input.energy));
            let line_count = input.map.lines_through(station.id).count();
            ScreenStation {
                id: station.id,
                pos: lens.apply(view),
                radius: STATION_RADIUS
                    * scale
                    * magnification.clamp(MIN_HIT_SCALE, MAX_LABEL_SCALE),
                magnification,
                energy: energy as f32,
                is_interchange: line_count >= 2,
                line_count,
                color: colors.get(&station.id).copied(),
                dimmed: input
                    .highlight
                    .is_some_and(|h| !on_line(input.map, h, station.id)),
                selected: input.selection.contains(&station.id),
                primary: input.primary == Some(station.id),
                hovered: input.hovered == Some(station.id),
            }
        })
        .collect()
}

/// The colour of the first line (in line order) through each station.
fn first_line_colors(map: &MetroMap) -> BTreeMap<StationId, Color32> {
    let mut colors = BTreeMap::new();
    for line in map.ordered_lines() {
        for &id in &line.stations {
            colors.entry(id).or_insert_with(|| line_color(map, line));
        }
    }
    colors
}

/// Label level of detail: show it if magnified enough, selected, hovered, or hot.
pub(super) fn build_labels(input: &PlanInput, stations: &[ScreenStation]) -> Vec<ScreenLabel> {
    stations
        .iter()
        .filter_map(|s| {
            let station = input.map.stations.get(&s.id)?;
            let hot = station.energy(input.now, input.energy) >= HOT_LABEL_ENERGY;
            let magnified = input.camera.zoom * s.magnification >= LABEL_THRESHOLD;
            if !(magnified || s.selected || s.hovered || hot) {
                return None;
            }
            let placeholder = station.is_untitled();
            Some(ScreenLabel {
                anchor: s.pos + Vec2::new(s.radius + 8.0, 0.0),
                text: station.display_title().to_owned(),
                size: BASE_FONT_PX * s.magnification.clamp(1.0, MAX_LABEL_SCALE),
                placeholder,
                emphasized: s.selected,
            })
        })
        .collect()
}

fn on_line(map: &MetroMap, line: LineId, station: StationId) -> bool {
    map.lines.get(&line).is_some_and(|l| l.contains(station))
}
