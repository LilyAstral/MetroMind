//! Bottom status line: the latest message (or a gentle hint), file state and
//! view readouts. Errors show here in amber; the app never panics on them.

use crate::render::Theme;
use eframe::egui::{self, Layout, RichText, Ui};
use std::time::{Duration, Instant};

/// How long a message stays before the hint returns.
pub const MESSAGE_LIFETIME: Duration = Duration::from_secs(6);
const IDLE_HINT: &str = "Double-click to add an idea · drag to move · F1 for shortcuts";

#[derive(Debug, Default)]
pub struct Status {
    message: Option<(String, Instant, bool)>,
}

impl Status {
    pub fn info(&mut self, text: impl Into<String>) {
        self.message = Some((text.into(), Instant::now(), false));
    }

    pub fn problem(&mut self, text: impl Into<String>) {
        self.message = Some((text.into(), Instant::now(), true));
    }

    /// The live message, if any, and how long until it expires.
    fn current(&self) -> Option<(&str, bool, Duration)> {
        let (text, at, problem) = self.message.as_ref()?;
        let left = MESSAGE_LIFETIME.checked_sub(at.elapsed())?;
        Some((text, *problem, left))
    }
}

pub struct StatusInfo {
    pub file: String,
    /// "saved", "saving…" or "not saved yet".
    pub save_state: &'static str,
    pub zoom: f32,
    pub lens: Option<(f32, f32)>,
}

pub fn show(ui: &mut Ui, status: &Status, info: &StatusInfo, theme: &Theme) {
    egui::Panel::bottom("status").show(ui, |ui| {
        ui.horizontal_centered(|ui| {
            match status.current() {
                Some((text, problem, left)) => {
                    let color = if problem { theme.notice } else { theme.text };
                    ui.label(RichText::new(text).color(color));
                    ui.ctx().request_repaint_after(left);
                }
                None => {
                    ui.label(RichText::new(IDLE_HINT).color(theme.text_dim));
                }
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(readout(info))
                    .on_hover_text("Changes save automatically");
            });
        });
    });
}

fn readout(info: &StatusInfo) -> String {
    let lens = match info.lens {
        Some((strength, radius)) => format!("lens ×{:.1} · {radius:.0}px", strength + 1.0),
        None => "lens off".into(),
    };
    format!(
        "{} ({}) · zoom {:.0}% · {lens}",
        info.file,
        info.save_state,
        info.zoom * 100.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_replace_each_other() {
        let mut status = Status::default();
        status.info("one");
        status.problem("two");
        let (text, problem, _) = status.current().expect("fresh message");
        assert_eq!((text, problem), ("two", true));
    }

    #[test]
    fn readout_mentions_lens_state() {
        let info = StatusInfo {
            file: "Ideas".into(),
            save_state: "saved",
            zoom: 1.0,
            lens: None,
        };
        assert!(readout(&info).contains("lens off"));
    }
}
