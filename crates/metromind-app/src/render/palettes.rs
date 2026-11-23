//! The six line palettes. Each has nine colours, one per number key. Every
//! colour must read on the dark background and must not be red (see tests).

use eframe::egui::Color32;

pub struct Palette {
    pub name: &'static str,
    pub colors: [Color32; 9],
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

pub const PALETTES: [Palette; 6] = [
    Palette {
        name: "Pastel",
        colors: [
            rgb(142, 197, 232),
            rgb(244, 177, 131),
            rgb(200, 162, 124),
            rgb(143, 209, 193),
            rgb(185, 169, 230),
            rgb(235, 213, 142),
            rgb(168, 221, 160),
            rgb(169, 184, 216),
            rgb(227, 201, 168),
        ],
    },
    Palette {
        name: "Harbour",
        colors: [
            rgb(126, 180, 226),
            rgb(160, 206, 236),
            rgb(98, 170, 196),
            rgb(140, 214, 214),
            rgb(186, 196, 232),
            rgb(118, 150, 206),
            rgb(204, 222, 240),
            rgb(214, 196, 160),
            rgb(170, 190, 170),
        ],
    },
    Palette {
        name: "Terracotta",
        colors: [
            rgb(232, 160, 112),
            rgb(246, 196, 150),
            rgb(196, 146, 104),
            rgb(224, 182, 128),
            rgb(250, 214, 170),
            rgb(214, 168, 140),
            rgb(186, 160, 128),
            rgb(236, 200, 120),
            rgb(206, 184, 160),
        ],
    },
    Palette {
        name: "Meadow",
        colors: [
            rgb(160, 210, 150),
            rgb(196, 226, 160),
            rgb(130, 196, 170),
            rgb(214, 220, 140),
            rgb(150, 200, 200),
            rgb(186, 210, 186),
            rgb(226, 206, 150),
            rgb(170, 186, 140),
            rgb(140, 180, 150),
        ],
    },
    Palette {
        name: "Dusk",
        colors: [
            rgb(190, 170, 232),
            rgb(214, 186, 230),
            rgb(160, 170, 226),
            rgb(200, 200, 240),
            rgb(176, 196, 230),
            rgb(226, 206, 236),
            rgb(150, 180, 210),
            rgb(210, 190, 210),
            rgb(186, 176, 206),
        ],
    },
    Palette {
        name: "High contrast",
        colors: [
            rgb(102, 194, 255),
            rgb(255, 184, 77),
            rgb(120, 220, 140),
            rgb(200, 160, 255),
            rgb(255, 230, 110),
            rgb(80, 220, 210),
            rgb(230, 230, 230),
            rgb(255, 150, 200),
            rgb(180, 200, 120),
        ],
    },
];

/// The palette at `index`, clamped so an unknown saved value falls back to the last.
pub fn palette(index: u8) -> &'static Palette {
    &PALETTES[usize::from(index).min(PALETTES.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Theme;

    /// WCAG relative luminance of an sRGB colour.
    fn luminance(c: Color32) -> f32 {
        let lin = |v: u8| {
            let s = f32::from(v) / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
    }

    fn contrast(a: Color32, b: Color32) -> f32 {
        let (hi, lo) = {
            let (x, y) = (luminance(a), luminance(b));
            (x.max(y), x.min(y))
        };
        (hi + 0.05) / (lo + 0.05)
    }

    /// Hue in degrees (0..360) and HSV saturation (0..1).
    fn hue_and_saturation(c: Color32) -> (f32, f32) {
        let (r, g, b) = (f32::from(c.r()), f32::from(c.g()), f32::from(c.b()));
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        if delta == 0.0 {
            return (0.0, 0.0);
        }
        let hue = if max == r {
            60.0 * ((g - b) / delta).rem_euclid(6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };
        (hue, delta / max)
    }

    #[test]
    fn every_colour_contrasts_with_the_background() {
        let background = Theme::default().background;
        for palette in &PALETTES {
            for &color in &palette.colors {
                let ratio = contrast(color, background);
                assert!(
                    ratio >= 3.0,
                    "{} {color:?} has contrast {ratio:.2}",
                    palette.name
                );
            }
        }
    }

    #[test]
    fn no_colour_is_red() {
        for palette in &PALETTES {
            for &color in &palette.colors {
                let (hue, saturation) = hue_and_saturation(color);
                let near_red = hue <= 15.0 || hue >= 345.0;
                assert!(
                    !(near_red && saturation > 0.35),
                    "{} {color:?} reads as red (hue {hue:.0}, saturation {saturation:.2})",
                    palette.name
                );
            }
        }
    }

    #[test]
    fn unknown_palette_falls_back_to_the_last() {
        assert_eq!(palette(200).name, "High contrast");
        assert_eq!(palette(0).name, "Pastel");
    }
}
