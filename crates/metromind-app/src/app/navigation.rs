//! Finding and jumping to ideas: the search box, the camera glide to a match,
//! and the hover preview of a station's notes.

use super::MetroMindApp;
use crate::frame_plan::to_vec2;
use crate::input::Command;
use crate::ui::hover_panel::{self, Card, HOVER_DELAY};
use crate::ui::search::{self, Action, SearchState};
use eframe::egui::{Context, Rect};
use metromind_core::chrono::{DateTime, Utc};
use metromind_core::clock::Clock;
use metromind_core::geometry::Camera;
use metromind_core::model::StationId;

/// How long the camera takes to glide to a search result.
const GLIDE_SECONDS: f64 = 0.2;

/// A camera move in progress. The start time is egui's clock, in seconds.
#[derive(Debug, Clone, Copy)]
pub struct CameraGlide {
    pub from: Camera,
    pub to: Camera,
    pub started: DateTime<Utc>,
}

impl MetroMindApp {
    pub(super) fn open_search(&mut self) {
        self.search = Some(SearchState::new());
    }

    /// Draw the search box, and queue a jump or close if the player chose one.
    pub(super) fn show_search(&mut self, ctx: &Context, viewport: Rect) {
        let Some(state) = self.search.as_mut() else {
            return;
        };
        let command = match search::show(ctx, state, &self.doc.map, viewport) {
            Some(Action::Jump(id)) => Command::JumpTo(id),
            Some(Action::Close) => {
                self.search = None;
                return;
            }
            None => return,
        };
        self.queued_commands.push(command);
    }

    /// Centre the view on `id`, keep the zoom, move the lens there, and select it.
    /// With reduce motion on the camera jumps; otherwise it glides.
    pub(super) fn jump_to(&mut self, id: StationId, viewport: Rect) {
        let Some(station) = self.doc.map.stations.get(&id) else {
            return;
        };
        let centre = to_vec2(viewport.center());
        let target = self.settings.camera.centered_on(station.position, centre);
        if self.settings.reduce_motion {
            self.settings.camera = target;
        } else {
            self.glide = Some(CameraGlide {
                from: self.settings.camera,
                to: target,
                started: self.clock.now(),
            });
        }
        self.settings.lens.focus = centre;
        self.search = None;
        self.select_only(Some(id));
    }

    /// Step the glide along an ease-out curve, asking for repaints only while it runs.
    pub(super) fn advance_glide(&mut self, ctx: &Context) {
        let Some(glide) = self.glide else {
            return;
        };
        let elapsed = (self.clock.now() - glide.started).num_milliseconds() as f64 / 1000.0;
        let t: f32 = (elapsed / GLIDE_SECONDS).clamp(0.0, 1.0) as f32;
        let eased = 1.0 - (1.0 - t).powi(2);
        self.settings.camera = lerp_camera(glide.from, glide.to, eased);
        if t >= 1.0 {
            self.glide = None;
        } else {
            ctx.request_repaint();
        }
    }

    /// Show the hover panel once the pointer has rested on a station. Once it
    /// is open, moving straight onto another station swaps it at once.
    pub(super) fn update_hover_panel(&mut self, ctx: &Context, now: f64, viewport: Rect) {
        let blocked = self.interaction.drag.is_some()
            || self.interaction.picker.is_some()
            || self.search.is_some()
            || self.rename.is_some()
            || ctx.any_popup_open();
        let hovered = self.interaction.hovered.filter(|_| !blocked);
        let Some(id) = hovered else {
            self.hover_since = None;
            hover_panel::hide(ctx);
            return;
        };
        let since = match self.hover_since {
            Some((seen, since)) if seen == id => since,
            Some((_, since)) if now - since >= HOVER_DELAY => now - HOVER_DELAY,
            _ => now,
        };
        self.hover_since = Some((id, since));
        let elapsed = now - since;
        let pointer = ctx.input(|i| i.pointer.hover_pos());
        let card = self
            .doc
            .map
            .stations
            .get(&id)
            .and_then(|station| Card::of(&self.doc.map, station));
        match (elapsed >= HOVER_DELAY, pointer, card) {
            (true, Some(pointer), Some(card)) => {
                let reduce = self.settings.reduce_motion;
                hover_panel::show(ctx, pointer, viewport, &card, reduce);
            }
            _ => {
                hover_panel::hide(ctx);
                if elapsed < HOVER_DELAY {
                    let wait = std::time::Duration::from_secs_f64(HOVER_DELAY - elapsed);
                    ctx.request_repaint_after(wait);
                }
            }
        }
    }
}

fn lerp_camera(from: Camera, to: Camera, t: f32) -> Camera {
    Camera {
        pan: from.pan + (to.pan - from.pan) * t,
        zoom: from.zoom + (to.zoom - from.zoom) * t,
    }
}
