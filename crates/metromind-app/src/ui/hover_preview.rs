//! A small tooltip with a station's notes, shown after the pointer has rested
//! on the station for a moment.

use eframe::egui::{Area, Context, Frame, Id, Order, Pos2, RichText, vec2};

/// How long the pointer must rest on a station before its notes appear, in seconds.
pub const HOVER_DELAY: f64 = 0.4;
const MAX_LINES: usize = 3;
const MAX_CHARS: usize = 160;
const OFFSET: f32 = 16.0;

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

/// Draw the tooltip just below and to the right of the pointer.
pub fn show(ctx: &Context, pointer: Pos2, title: &str, text: &str) {
    Area::new(Id::unique("hover_preview"))
        .order(Order::Tooltip)
        .fixed_pos(pointer + vec2(OFFSET, OFFSET))
        .interactable(false)
        .show(ctx, |ui| {
            Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(320.0);
                ui.label(RichText::new(title).strong());
                ui.label(text);
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_notes_show_nothing() {
        assert_eq!(preview_text("   \n "), None);
    }

    #[test]
    fn short_notes_are_shown_whole() {
        assert_eq!(preview_text("one\ntwo"), Some("one\ntwo".into()));
    }

    #[test]
    fn more_than_three_lines_are_cut_with_an_ellipsis() {
        assert_eq!(preview_text("a\nb\nc\nd"), Some("a\nb\nc…".into()));
    }

    #[test]
    fn long_notes_are_cut_at_160_characters() {
        let long = "x".repeat(200);
        let text = preview_text(&long).expect("has notes");
        assert_eq!(text.chars().count(), MAX_CHARS + 1);
        assert!(text.ends_with('…'));
    }
}
