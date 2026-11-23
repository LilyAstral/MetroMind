use super::{Command, InputContext, InputOutcome, StationDrag};
use crate::frame_plan::to_vec2;
use eframe::egui::{Context, Event, MouseWheelUnit, PointerButton, Pos2, Response};
use metromind_core::Vec2;
use metromind_core::editing::{Edit, Membership};
use metromind_core::geometry::{GRID_CELL, snap_to_grid};
use metromind_core::model::{Station, StationId};

/// Zoom factor per wheel notch.
const WHEEL_ZOOM_BASE: f32 = 1.12;
/// Pixel-precise wheels (touchpads) report points; this many count as one notch.
const POINTS_PER_NOTCH: f32 = 50.0;
const NOTCHES_PER_PAGE: f32 = 10.0;
const LENS_STRENGTH_PER_NOTCH: f32 = 0.25;
const LENS_RADIUS_PER_NOTCH: f32 = 20.0;

pub fn collect(ctx: &Context, response: &Response, cx: &InputContext, out: &mut InputOutcome) {
    let pointer = response.hover_pos();
    out.hovered = pointer.and_then(|p| cx.plan.station_at(p));
    clicks(response, cx, out);
    drags(ctx, response, cx, out);
    if response.contains_pointer() {
        wheel(ctx, pointer, out);
    }
}

fn clicks(response: &Response, cx: &InputContext, out: &mut InputOutcome) {
    let Some(p) = response.interact_pointer_pos() else {
        return;
    };
    if response.double_clicked() {
        double_click(p, cx, out);
    } else if response.clicked() {
        out.commands.push(Command::Select(cx.plan.station_at(p)));
    }
}

/// On a station: rename it. On a segment: insert a new station there.
/// On empty space: drop a new idea. Either way you can type right away.
fn double_click(p: Pos2, cx: &InputContext, out: &mut InputOutcome) {
    if let Some(id) = cx.plan.station_at(p) {
        let text = cx
            .map
            .stations
            .get(&id)
            .map(|s| s.title.clone())
            .unwrap_or_default();
        out.commands.push(Command::StartRename { id, text });
        return;
    }
    let world = snap_to_grid(cx.plan.camera.view_to_world(to_vec2(p)), GRID_CELL);
    let memberships = cx
        .plan
        .segment_at(p)
        .map(|l| Membership {
            line: l.line,
            index: l.insert_at,
        })
        .into_iter()
        .collect();
    let station = Station::new(StationId(cx.map.peek_id()), "", world, cx.now, cx.energy);
    let id = station.id;
    out.edits.push(Edit::AddStation {
        station,
        memberships,
    });
    out.commands.push(Command::Select(Some(id)));
    out.commands.push(Command::StartRename {
        id,
        text: String::new(),
    });
}

/// Left-drag a station to move it (one edit on release, snapped to the
/// grid). Any other drag pans the view.
fn drags(ctx: &Context, response: &Response, cx: &InputContext, out: &mut InputOutcome) {
    if let Some(drag) = cx.interaction.drag {
        continue_drag(drag, response, cx, out);
        return;
    }
    if response.drag_started_by(PointerButton::Primary) {
        let origin = ctx.input(|i| i.pointer.press_origin());
        if let Some(drag) = origin.and_then(|o| start_drag(o, cx)) {
            out.commands.push(Command::Select(Some(drag.id)));
            out.commands.push(Command::Drag(Some(drag)));
            return;
        }
    }
    if response.dragged() {
        let delta = response.drag_delta();
        out.commands.push(Command::Pan(Vec2::new(delta.x, delta.y)));
    }
}

fn start_drag(origin: Pos2, cx: &InputContext) -> Option<StationDrag> {
    let id = cx.plan.station_at(origin)?;
    let position = cx.map.stations.get(&id)?.position;
    let grabbed = cx.plan.camera.view_to_world(to_vec2(origin));
    Some(StationDrag {
        id,
        offset: position - grabbed,
        world: position,
        start: position,
    })
}

fn continue_drag(
    drag: StationDrag,
    response: &Response,
    cx: &InputContext,
    out: &mut InputOutcome,
) {
    let pointer = response.interact_pointer_pos();
    match pointer {
        Some(p) if response.dragged() && !response.drag_stopped() => {
            let world = cx.plan.camera.view_to_world(to_vec2(p)) + drag.offset;
            out.commands
                .push(Command::Drag(Some(StationDrag { world, ..drag })));
        }
        _ => {
            let to = snap_to_grid(drag.world, GRID_CELL);
            if to != drag.start {
                out.edits.push(Edit::MoveStation { id: drag.id, to });
            }
            out.commands.push(Command::Drag(None));
        }
    }
}

/// Wheel: zoom. Ctrl+wheel: lens strength. Shift+wheel: lens radius.
/// Read from raw events because egui rewrites Ctrl/Shift+wheel for itself.
fn wheel(ctx: &Context, pointer: Option<Pos2>, out: &mut InputOutcome) {
    ctx.input(|i| {
        for event in &i.events {
            let Event::MouseWheel {
                unit,
                delta,
                modifiers,
                ..
            } = event
            else {
                continue;
            };
            let raw = if delta.x.abs() > delta.y.abs() {
                delta.x
            } else {
                delta.y
            };
            let notches = match unit {
                MouseWheelUnit::Point => raw / POINTS_PER_NOTCH,
                MouseWheelUnit::Line => raw,
                MouseWheelUnit::Page => raw * NOTCHES_PER_PAGE,
            };
            let command = if modifiers.command || modifiers.ctrl {
                Command::AdjustLensStrength(notches * LENS_STRENGTH_PER_NOTCH)
            } else if modifiers.shift {
                Command::AdjustLensRadius(notches * LENS_RADIUS_PER_NOTCH)
            } else if let Some(p) = pointer {
                Command::Zoom {
                    anchor: to_vec2(p),
                    factor: WHEEL_ZOOM_BASE.powf(notches),
                }
            } else {
                continue;
            };
            out.commands.push(command);
        }
    });
}
