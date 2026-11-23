//! The central map: input → edits/commands → plan → render, every frame.

use super::MetroMindApp;
use crate::frame_plan::{self, FramePlan, PlanInput, to_vec2};
use crate::input::{self, InputContext};
use crate::render::{self, Textures, field::FieldKey};
use crate::ui;
use eframe::egui::{self, Context, CursorIcon, Rect, Sense, Ui};
use metromind_core::clock::Clock;
use metromind_core::editing::Edit;

/// Fraction of the remaining distance the lens focus moves each frame.
const LENS_SMOOTHING: f32 = 0.25;
/// Below this distance (px) the lens counts as caught up and repaints stop.
const LENS_SETTLED_PX: f32 = 0.5;

impl MetroMindApp {
    pub(super) fn map_view(&mut self, ui: &mut Ui) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let ctx = ui.ctx().clone();
        let now = self.clock.now();
        self.interaction.typing = self.rename.is_some();
        let outcome = input::collect(
            &ctx,
            &response,
            &InputContext {
                plan: &self.plan,
                map: &self.doc.map,
                interaction: self.interaction.clone(),
                now,
                energy: &self.settings.energy,
            },
        );
        self.interaction.hovered = outcome.hovered;
        self.lens_pinned = outcome.pin_lens;
        self.queued_edits.extend(outcome.edits);
        self.queued_commands.extend(outcome.commands);
        if std::mem::take(&mut self.fit_pending) {
            self.fit(rect);
        }
        self.show_picker(&ctx, rect);
        self.apply_queued(rect);
        self.follow_cursor(&ctx, response.hover_pos());
        self.plan = self.build_plan(rect);
        self.draw(ui, rect);
        self.set_cursor(&ctx, &response);
    }

    fn apply_queued(&mut self, viewport: Rect) {
        for edit in std::mem::take(&mut self.queued_edits) {
            self.apply_edit(edit);
        }
        for command in std::mem::take(&mut self.queued_commands) {
            self.run_command(command, viewport);
        }
    }

    pub(super) fn apply_edit(&mut self, edit: Edit) {
        if let Err(err) = self.doc.apply(edit, &self.clock, &self.settings.energy) {
            self.status.problem(err.to_string());
        }
        self.forget_missing_selection();
    }

    /// Exponentially smooth the lens focus toward the cursor, repainting only
    /// until it has caught up. Space pins it.
    fn follow_cursor(&mut self, ctx: &Context, pointer: Option<egui::Pos2>) {
        let Some(target) = pointer.map(to_vec2) else {
            return;
        };
        if !self.settings.lens_enabled || self.lens_pinned {
            return;
        }
        let focus = &mut self.settings.lens.focus;
        if self.settings.reduce_motion || focus.distance(target) <= LENS_SETTLED_PX {
            *focus = target;
            return;
        }
        *focus += (target - *focus) * LENS_SMOOTHING;
        ctx.request_repaint();
    }

    fn build_plan(&self, viewport: Rect) -> FramePlan {
        frame_plan::build(&PlanInput {
            map: &self.doc.map,
            camera: self.settings.camera,
            lens: self.settings.lens,
            lens_enabled: self.settings.lens_enabled,
            now: self.clock.now(),
            energy: &self.settings.energy,
            viewport,
            selection: &self.interaction.selection,
            primary: self.interaction.primary,
            hovered: self.interaction.hovered,
            drag: self.interaction.drag.as_ref(),
            marquee: self
                .interaction
                .marquee
                .map(|(a, b)| Rect::from_two_pos(a, b)),
            connect: self
                .interaction
                .connect
                .as_ref()
                .map(|c| (c.from, c.pointer)),
        })
    }

    fn draw(&mut self, ui: &mut Ui, rect: Rect) {
        let field = self.settings.layers.field.then(|| {
            let key = FieldKey {
                revision: self.doc.revision,
                minute: self.clock.now().timestamp() / 60,
                camera: self.plan.camera,
                lens: self.plan.lens,
                viewport: rect,
                drag: self.interaction.drag.as_ref().map(|d| d.delta),
            };
            self.field.texture_for(ui.ctx(), &self.plan, key)
        });
        let textures = Textures {
            grain: self.grain.id(),
            field,
        };
        let painter = ui.painter_at(rect);
        render::draw_all(
            &painter,
            &self.plan,
            self.settings.layers,
            &self.theme,
            textures,
        );
        if self.doc.map.stations.is_empty() {
            ui::draw_empty_hint(&painter, rect, &self.theme);
        }
    }

    fn set_cursor(&self, ctx: &Context, response: &egui::Response) {
        if self.interaction.drag.is_some() {
            ctx.set_cursor_icon(CursorIcon::Grabbing);
        } else if self.interaction.hovered.is_some() {
            ctx.set_cursor_icon(CursorIcon::PointingHand);
        } else if response.dragged() {
            ctx.set_cursor_icon(CursorIcon::Move);
        }
    }
}
