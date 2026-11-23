//! Drawing. A layer is a plain function that reads the [`FramePlan`]; layers
//! never touch the map.

mod background;
pub mod colors;
pub mod field;
mod ghosts;
mod glow;
mod grid;
mod halos;
mod labels;
mod lens_ring;
mod lines;
mod marquee;
mod stations;

pub use background::grain_image;
pub use colors::Theme;

use crate::frame_plan::FramePlan;
use eframe::egui::{Painter, TextureId};
use serde::{Deserialize, Serialize};

/// The independently toggleable render passes, in draw order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Grid,
    Field,
    Halos,
    Lines,
    Stations,
    Labels,
    LensRing,
}

impl Layer {
    pub const ALL: [Layer; 7] = [
        Layer::Grid,
        Layer::Field,
        Layer::Halos,
        Layer::Lines,
        Layer::Stations,
        Layer::Labels,
        Layer::LensRing,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Layer::Grid => "Grid",
            Layer::Field => "Energy field",
            Layer::Halos => "Halos",
            Layer::Lines => "Lines",
            Layer::Stations => "Stations",
            Layer::Labels => "Labels",
            Layer::LensRing => "Lens ring",
        }
    }
}

/// Which layers are on. Saved in the app settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayerVisibility {
    pub grid: bool,
    pub field: bool,
    pub halos: bool,
    pub lines: bool,
    pub stations: bool,
    pub labels: bool,
    pub lens_ring: bool,
}

impl Default for LayerVisibility {
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

impl LayerVisibility {
    pub fn flag_mut(&mut self, layer: Layer) -> &mut bool {
        match layer {
            Layer::Grid => &mut self.grid,
            Layer::Field => &mut self.field,
            Layer::Halos => &mut self.halos,
            Layer::Lines => &mut self.lines,
            Layer::Stations => &mut self.stations,
            Layer::Labels => &mut self.labels,
            Layer::LensRing => &mut self.lens_ring,
        }
    }

    pub fn is_on(mut self, layer: Layer) -> bool {
        *self.flag_mut(layer)
    }
}

/// GPU textures the layers draw with; owned by the app.
#[derive(Debug, Clone, Copy)]
pub struct Textures {
    pub grain: TextureId,
    /// The cached energy-field heatmap, when the Field layer is on.
    pub field: Option<TextureId>,
}

/// Draw every visible layer in the fixed order.
pub fn draw_all(
    painter: &Painter,
    plan: &FramePlan,
    layers: LayerVisibility,
    theme: &Theme,
    textures: Textures,
) {
    background::draw_background(painter, plan, theme, textures.grain);
    for layer in Layer::ALL.into_iter().filter(|&l| layers.is_on(l)) {
        match layer {
            Layer::Grid => grid::draw_grid(painter, plan, theme),
            Layer::Field => {
                if let Some(texture) = textures.field {
                    field::draw_field(painter, plan, texture);
                }
            }
            Layer::Halos => halos::draw_halos(painter, plan, theme),
            Layer::Lines => lines::draw_lines(painter, plan, theme),
            Layer::Stations => {
                ghosts::draw_ghosts(painter, plan, theme);
                stations::draw_stations(painter, plan, theme);
            }
            Layer::Labels => labels::draw_labels(painter, plan, theme),
            Layer::LensRing => lens_ring::draw_lens_ring(painter, plan, theme),
        }
    }
    marquee::draw_marquee(painter, plan, theme);
}
