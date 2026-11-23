use super::Theme;
use super::colors::{energy_color, with_alpha};
use super::glow::radial_glow;
use crate::frame_plan::{FramePlan, STATION_RADIUS, to_pos2};
use eframe::egui::{Mesh, Painter, Shape};

/// Energy glow beneath each station:
/// radius = `STATION_RADIUS × (1.5 + 3 × E)` and center alpha = `0.15 + 0.55 × E`,
/// both scaled by the lens magnification. All halos go into one mesh so 500
/// stations cost a single draw call.
pub fn draw_halos(painter: &Painter, plan: &FramePlan, _theme: &Theme) {
    let mut mesh = Mesh::default();
    for station in &plan.stations {
        let e = station.energy;
        let radius = STATION_RADIUS * (1.5 + 3.0 * e) * station.magnification;
        let color = with_alpha(energy_color(e), 0.15 + 0.55 * e);
        mesh.append(radial_glow(to_pos2(station.pos), radius, color));
    }
    painter.add(Shape::mesh(mesh));
}
