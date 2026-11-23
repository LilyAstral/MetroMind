//! Palette and theme. Dark, calm, pastel; red never appears, because red reads
//! as an error or "overdue" — exactly the shame signal MetroMind avoids.

use eframe::egui::Color32;

/// Pastel line colors, one per number key. Blues, browns and oranges first.
pub const LINE_COLORS: [Color32; 9] = [
    Color32::from_rgb(142, 197, 232), // pastel blue
    Color32::from_rgb(244, 177, 131), // peach orange
    Color32::from_rgb(200, 162, 124), // tan brown
    Color32::from_rgb(143, 209, 193), // sage teal
    Color32::from_rgb(185, 169, 230), // lavender
    Color32::from_rgb(235, 213, 142), // butter
    Color32::from_rgb(168, 221, 160), // mint
    Color32::from_rgb(169, 184, 216), // steel blue
    Color32::from_rgb(227, 201, 168), // sand
];

/// Energy ramp stops, cool → warm: slate blue (cold) → teal → amber (hot).
const ENERGY_STOPS: [(f32, Color32); 3] = [
    (0.0, Color32::from_rgb(107, 127, 168)),
    (0.5, Color32::from_rgb(79, 179, 169)),
    (1.0, Color32::from_rgb(242, 180, 90)),
];

pub fn line_color(index: u8) -> Color32 {
    LINE_COLORS[usize::from(index) % LINE_COLORS.len()]
}

/// Color temperature for energy `e` in 0..=1 (clamped). Never red.
pub fn energy_color(e: f32) -> Color32 {
    let e = e.clamp(0.0, 1.0);
    let upper = ENERGY_STOPS
        .iter()
        .position(|&(at, _)| at >= e)
        .unwrap_or(ENERGY_STOPS.len() - 1)
        .max(1);
    let (a_at, a) = ENERGY_STOPS[upper - 1];
    let (b_at, b) = ENERGY_STOPS[upper];
    lerp_color(a, b, (e - a_at) / (b_at - a_at))
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let mix = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgb(mix(a.r(), b.r()), mix(a.g(), b.g()), mix(a.b(), b.b()))
}

/// `color` with an alpha in 0..=1, ready for egui's premultiplied pipeline.
pub fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}

/// Every non-line color the map uses, in one place.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Color32,
    pub grid_dot: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub label_plate: Color32,
    pub station_fill: Color32,
    pub interchange_fill: Color32,
    pub interchange_outline: Color32,
    pub free_station: Color32,
    pub selection: Color32,
    pub hover: Color32,
    pub lens_ring: Color32,
    /// Friendly attention color for status messages. Amber, not red.
    pub notice: Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color32::from_rgb(27, 30, 36),
            grid_dot: Color32::from_rgba_unmultiplied(150, 162, 182, 70),
            text: Color32::from_rgb(237, 230, 218),
            text_dim: Color32::from_rgb(168, 174, 186),
            label_plate: Color32::from_rgba_unmultiplied(22, 24, 30, 225),
            station_fill: Color32::from_rgb(34, 38, 46),
            interchange_fill: Color32::from_rgb(247, 244, 238),
            interchange_outline: Color32::from_rgb(35, 38, 45),
            free_station: Color32::from_rgb(196, 202, 214),
            selection: Color32::from_rgb(220, 235, 250),
            hover: Color32::from_rgba_unmultiplied(220, 235, 250, 110),
            lens_ring: Color32::from_rgba_unmultiplied(200, 215, 235, 70),
            notice: Color32::from_rgb(232, 184, 109),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_reddish(c: Color32) -> bool {
        let (r, g, b) = (i32::from(c.r()), i32::from(c.g()), i32::from(c.b()));
        r > 150 && r - g > 70 && r - b > 70
    }

    #[test]
    fn energy_ramp_hits_its_stops() {
        assert_eq!(energy_color(0.0), ENERGY_STOPS[0].1);
        assert_eq!(energy_color(0.5), ENERGY_STOPS[1].1);
        assert_eq!(energy_color(1.0), ENERGY_STOPS[2].1);
        assert_eq!(energy_color(7.0), ENERGY_STOPS[2].1);
    }

    #[test]
    fn no_red_anywhere() {
        let ramp = (0..=100).map(|i| energy_color(i as f32 / 100.0));
        assert!(ramp.chain(LINE_COLORS).all(|c| !is_reddish(c)));
    }

    #[test]
    fn hot_is_warmer_than_cold() {
        let (cold, hot) = (energy_color(0.0), energy_color(1.0));
        assert!(hot.r() > cold.r() && hot.b() < cold.b());
    }

    #[test]
    fn line_colors_wrap() {
        assert_eq!(line_color(9), line_color(0));
    }
}
