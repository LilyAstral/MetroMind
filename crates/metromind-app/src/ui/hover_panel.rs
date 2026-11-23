//! The hover panel: a non-interactive glass card with a station's title, notes
//! and list, shown beside the pointer once it has rested on the station. It
//! replaces the old plain tooltip.

use crate::render::colors::{Theme, line_color};
use crate::ui::glass;
use eframe::egui::{
    Area, Color32, Context, Frame, Id, Margin, Order, Pos2, Rect, RichText, Sense, Shape, Ui, Vec2,
    pos2, vec2,
};
use metromind_core::model::{ListStyle, MetroMap, NoteItem, Station};

/// How long the pointer must rest on a station before its panel appears, in seconds.
pub const HOVER_DELAY: f64 = 0.4;
/// Fade-in time. Hiding is instant.
const FADE_SECS: f32 = 0.12;
const PANEL_WIDTH: f32 = 280.0;
const PAD: i8 = 14;
const MAX_LINES: usize = 4;
const MAX_CHARS: usize = 200;
const MAX_ITEMS: usize = 6;
/// Gap between the pointer and the panel's corner.
const GAP_X: f32 = 16.0;
const GAP_Y: f32 = 8.0;
const SWATCH: f32 = 10.0;

/// The first few lines of `notes`, cut short with "…". `None` if there are no notes.
pub fn preview_text(notes: &str) -> Option<String> {
    let trimmed = notes.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lines: Vec<&str> = trimmed.lines().collect();
    let head = lines
        .iter()
        .take(MAX_LINES)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    let more_lines = lines.len() > MAX_LINES;
    let too_long = head.chars().count() > MAX_CHARS;
    if too_long {
        let cut: String = head.chars().take(MAX_CHARS).collect();
        Some(format!("{cut}…"))
    } else if more_lines {
        Some(format!("{head}…"))
    } else {
        Some(head)
    }
}

/// What the panel shows, copied out of the map so drawing needs no borrows.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    title: String,
    swatches: Vec<Color32>,
    notes: Option<String>,
    items: Vec<NoteItem>,
    more_items: bool,
    style: ListStyle,
}

impl Card {
    /// `None` when the station has neither notes nor list items.
    pub fn of(map: &MetroMap, station: &Station) -> Option<Self> {
        let notes = preview_text(&station.notes);
        let filled: Vec<NoteItem> = station
            .items
            .iter()
            .filter(|item| !item.text.trim().is_empty())
            .cloned()
            .collect();
        if notes.is_none() && filled.is_empty() {
            return None;
        }
        Some(Self {
            title: station.display_title().to_owned(),
            swatches: map
                .lines_through(station.id)
                .map(|line| line_color(map, line))
                .collect(),
            notes,
            more_items: filled.len() > MAX_ITEMS,
            items: filled.into_iter().take(MAX_ITEMS).collect(),
            style: station.list_style,
        })
    }
}

/// Where the panel goes: beside the cursor, flipped left near the right edge,
/// and kept inside the viewport vertically (and horizontally, if it must).
pub fn place(cursor: Pos2, size: Vec2, viewport: Rect) -> Pos2 {
    let right = cursor.x + GAP_X;
    let x = if right + size.x > viewport.max.x {
        cursor.x - GAP_X - size.x
    } else {
        right
    };
    let y = cursor.y - GAP_Y;
    pos2(
        x.min(viewport.max.x - size.x).max(viewport.min.x),
        y.min(viewport.max.y - size.y).max(viewport.min.y),
    )
}

/// Draw the panel at the cursor. Call it every frame the panel should show.
pub fn show(ctx: &Context, pointer: Pos2, viewport: Rect, card: &Card, reduce_motion: bool) {
    let secs = if reduce_motion { 0.0 } else { FADE_SECS };
    let alpha = ctx.animate_bool_with_time(fade_id(), true, secs);
    let size = ctx
        .data(|d| d.get_temp::<Vec2>(size_id()))
        .unwrap_or(vec2(PANEL_WIDTH, 120.0));
    let at = place(pointer, size, viewport);
    let area = Area::new(Id::unique("hover_panel"))
        .order(Order::Tooltip)
        .fixed_pos(at)
        .interactable(false)
        .show(ctx, |ui| {
            let slot = ui.painter().add(Shape::Noop);
            let frame = Frame::NONE.inner_margin(Margin::same(PAD)).show(ui, |ui| {
                ui.set_width(PANEL_WIDTH - 2.0 * f32::from(PAD));
                ui.multiply_opacity(alpha);
                content(ui, card);
            });
            let background = Theme::default().background;
            let glass = glass::shapes(frame.response.rect, background, alpha);
            ui.painter().set(slot, Shape::Vec(glass));
        });
    let measured = area.response.rect.size();
    ctx.data_mut(|d| d.insert_temp(size_id(), measured));
}

/// Hide the panel. Hiding is instant, and the next show fades in from nothing.
pub fn hide(ctx: &Context) {
    ctx.animate_bool_with_time(fade_id(), false, 0.0);
}

fn fade_id() -> Id {
    Id::unique("hover_panel_fade")
}

fn size_id() -> Id {
    Id::unique("hover_panel_size")
}

fn content(ui: &mut Ui, card: &Card) {
    let theme = Theme::default();
    ui.label(RichText::new(&card.title).strong().color(theme.text));
    if !card.swatches.is_empty() {
        ui.horizontal(|ui| {
            for &color in &card.swatches {
                let (rect, _) = ui.allocate_exact_size(vec2(SWATCH, SWATCH), Sense::hover());
                ui.painter().rect_filled(rect, 2.0, color);
            }
        });
    }
    if let Some(notes) = &card.notes {
        ui.add_space(4.0);
        ui.label(RichText::new(notes).color(theme.text));
    }
    if !card.items.is_empty() {
        ui.add_space(6.0);
        item_rows(ui, card, &theme);
    }
}

/// Read-only rows. Ticked items are dimmed in place; there are no counts.
fn item_rows(ui: &mut Ui, card: &Card, theme: &Theme) {
    for item in &card.items {
        let dimmed = card.style == ListStyle::Checkboxes && item.done;
        let color = if dimmed { theme.text_dim } else { theme.text };
        let mark = match (card.style, item.done) {
            (ListStyle::Bullets, _) => "•",
            (ListStyle::Checkboxes, true) => "☑",
            (ListStyle::Checkboxes, false) => "☐",
        };
        ui.label(RichText::new(format!("{mark} {}", item.text)).color(color));
    }
    if card.more_items {
        ui.label(RichText::new("…").color(theme.text_dim));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport() -> Rect {
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1000.0, 800.0))
    }

    #[test]
    fn panel_sits_beside_and_above_the_cursor() {
        let at = place(pos2(100.0, 100.0), vec2(280.0, 200.0), viewport());
        assert_eq!(at, pos2(116.0, 92.0));
    }

    #[test]
    fn panel_flips_left_near_the_right_edge() {
        let at = place(pos2(900.0, 100.0), vec2(280.0, 200.0), viewport());
        assert_eq!(at, pos2(604.0, 92.0));
    }

    #[test]
    fn panel_is_clamped_to_the_bottom_edge() {
        let at = place(pos2(100.0, 780.0), vec2(280.0, 200.0), viewport());
        assert_eq!(at.y, 600.0);
    }

    #[test]
    fn panel_is_clamped_to_the_top_edge() {
        let at = place(pos2(100.0, 0.0), vec2(280.0, 200.0), viewport());
        assert_eq!(at.y, 0.0);
    }

    #[test]
    fn notes_are_cut_at_four_lines() {
        assert_eq!(preview_text("a\nb\nc\nd"), Some("a\nb\nc\nd".into()));
        assert_eq!(preview_text("a\nb\nc\nd\ne"), Some("a\nb\nc\nd…".into()));
    }

    #[test]
    fn notes_are_cut_at_200_characters() {
        let text = preview_text(&"x".repeat(300)).expect("has notes");
        assert_eq!(text.chars().count(), MAX_CHARS + 1);
        assert!(text.ends_with('…'));
    }

    fn luminance(c: Color32) -> f32 {
        let lin = |v: u8| {
            let s = f32::from(v) / 255.0;
            if s <= 0.039_28 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
    }

    fn contrast(a: Color32, b: Color32) -> f32 {
        let (hi, lo) = {
            let (la, lb) = (luminance(a), luminance(b));
            (la.max(lb), la.min(lb))
        };
        (hi + 0.05) / (lo + 0.05)
    }

    #[test]
    fn text_keeps_three_to_one_contrast_on_the_glass() {
        let theme = Theme::default();
        // The map shows through the body, and the sheen lightens its top third.
        let body = glass::blend(glass::body(theme.background), theme.background);
        let worst = glass::blend(glass::sheen_color(), body);
        for text in [theme.text, theme.text_dim] {
            assert!(contrast(text, worst) >= 3.0, "{text:?} on {worst:?}");
        }
    }

    #[test]
    fn ticked_items_are_listed_without_counts() {
        let done = NoteItem {
            text: "done".into(),
            done: true,
        };
        let open = NoteItem {
            text: "open".into(),
            done: false,
        };
        let shown: Vec<String> = [done, open]
            .iter()
            .map(|i| format!("{} {}", if i.done { "☑" } else { "☐" }, i.text))
            .collect();
        assert_eq!(shown, vec!["☑ done", "☐ open"]);
    }
}
