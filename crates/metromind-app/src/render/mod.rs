//! Drawing. A view is a plain function that reads the [`FramePlan`]; views
//! never touch the map.

mod background;
pub mod colors;
mod connector;
pub mod field;
mod ghosts;
mod glow;
mod grid;
mod halos;
mod labels;
mod lens_ring;
mod lines;
mod marquee;
pub mod palettes;
mod stations;

pub use background::grain_image;
pub use colors::Theme;

use crate::frame_plan::FramePlan;
use eframe::egui::{Painter, TextureId};
use serde::{Deserialize, Serialize};

/// The independently toggleable render passes, in draw order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Grid,
    Field,
    Halos,
    Lines,
    Stations,
    Labels,
    LensRing,
}

impl View {
    pub const ALL: [View; 7] = [
        View::Grid,
        View::Field,
        View::Halos,
        View::Lines,
        View::Stations,
        View::Labels,
        View::LensRing,
    ];

    pub fn label(self) -> &'static str {
        match self {
            View::Grid => "Grid",
            View::Field => "Energy field",
            View::Halos => "Halos",
            View::Lines => "Lines",
            View::Stations => "Stations",
            View::Labels => "Labels",
            View::LensRing => "Lens ring",
        }
    }
}

/// Which views are on. Saved in the app settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewVisibility {
    pub grid: bool,
    pub field: bool,
    pub halos: bool,
    pub lines: bool,
    pub stations: bool,
    pub labels: bool,
    pub lens_ring: bool,
}

impl Default for ViewVisibility {
    fn default() -> Self {
        Self {
            grid: true,
            field: false,
            halos: true,
            lines: true,
            stations: true,
            labels: true,
            lens_ring: true,
        }
    }
}

impl ViewVisibility {
    pub fn flag_mut(&mut self, view: View) -> &mut bool {
        match view {
            View::Grid => &mut self.grid,
            View::Field => &mut self.field,
            View::Halos => &mut self.halos,
            View::Lines => &mut self.lines,
            View::Stations => &mut self.stations,
            View::Labels => &mut self.labels,
            View::LensRing => &mut self.lens_ring,
        }
    }

    pub fn is_on(mut self, view: View) -> bool {
        *self.flag_mut(view)
    }
}

/// GPU textures the views draw with; owned by the app.
#[derive(Debug, Clone, Copy)]
pub struct Textures {
    pub grain: TextureId,
    /// The cached energy-field heatmap, when the Field view is on.
    pub field: Option<TextureId>,
}

/// Draw every visible view in the fixed order.
pub fn draw_all(
    painter: &Painter,
    plan: &FramePlan,
    views: ViewVisibility,
    theme: &Theme,
    textures: Textures,
) {
    background::draw_background(painter, plan, theme, textures.grain);
    for view in View::ALL.into_iter().filter(|&l| views.is_on(l)) {
        match view {
            View::Grid => grid::draw_grid(painter, plan, theme),
            View::Field => {
                if let Some(texture) = textures.field {
                    field::draw_field(painter, plan, texture);
                }
            }
            View::Halos => halos::draw_halos(painter, plan, theme),
            View::Lines => lines::draw_lines(painter, plan, theme),
            View::Stations => {
                ghosts::draw_ghosts(painter, plan, theme);
                connector::draw_connector(painter, plan, theme);
                stations::draw_stations(painter, plan, theme);
            }
            View::Labels => labels::draw_labels(painter, plan, theme),
            View::LensRing => lens_ring::draw_lens_ring(painter, plan, theme),
        }
    }
    marquee::draw_marquee(painter, plan, theme);
}
