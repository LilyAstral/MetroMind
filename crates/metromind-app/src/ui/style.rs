//! Global egui style: dark, soft and rounded, with roomy 44 px click targets.

use eframe::egui::{Color32, Context, CornerRadius, FontId, Stroke, TextStyle, Theme, vec2};

const ROUNDING: u8 = 10;
const MIN_TARGET: f32 = 44.0;

const PANEL: Color32 = Color32::from_rgb(33, 37, 44);
const FIELD: Color32 = Color32::from_rgb(22, 24, 30);
const WIDGET: Color32 = Color32::from_rgb(46, 51, 60);
const WIDGET_HOVER: Color32 = Color32::from_rgb(58, 65, 77);
pub const ACCENT: Color32 = Color32::from_rgb(142, 197, 232);
const TEXT: Color32 = Color32::from_rgb(237, 230, 218);

pub fn apply(ctx: &Context) {
    ctx.set_theme(Theme::Dark);
    ctx.style_mut_of(Theme::Dark, |style| {
        style.spacing.interact_size = vec2(MIN_TARGET, MIN_TARGET);
        style.spacing.button_padding = vec2(12.0, 6.0);
        style.spacing.item_spacing = vec2(10.0, 8.0);
        for (text_style, size) in [
            (TextStyle::Body, 16.0),
            (TextStyle::Button, 16.0),
            (TextStyle::Heading, 21.0),
            (TextStyle::Small, 13.0),
        ] {
            style
                .text_styles
                .insert(text_style, FontId::proportional(size));
        }
        let v = &mut style.visuals;
        v.panel_fill = PANEL;
        v.window_fill = PANEL;
        v.extreme_bg_color = FIELD;
        v.override_text_color = Some(TEXT);
        v.selection.bg_fill = ACCENT.linear_multiply(0.45);
        v.selection.stroke = Stroke::new(1.5, ACCENT);
        v.hyperlink_color = ACCENT;
        v.window_corner_radius = CornerRadius::same(ROUNDING + 4);
        v.menu_corner_radius = CornerRadius::same(ROUNDING);
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.corner_radius = CornerRadius::same(ROUNDING);
        }
        v.widgets.inactive.weak_bg_fill = WIDGET;
        v.widgets.inactive.bg_fill = WIDGET;
        v.widgets.hovered.weak_bg_fill = WIDGET_HOVER;
        v.widgets.hovered.bg_fill = WIDGET_HOVER;
    });
}
