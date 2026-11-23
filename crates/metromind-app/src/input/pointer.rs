use super::{Command, InputContext, InputOutcome, StationDrag};
use crate::frame_plan::to_vec2;
use eframe::egui::{Context, Event, MouseWheelUnit, PointerButton, Pos2, Rect, Response};
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
    clicks(ctx, response, cx, out);
    drags(ctx, response, cx, out);
    if response.contains_pointer() {
        wheel(ctx, pointer, out);
    }
}

fn clicks(ctx: &Context, response: &Response, cx: &InputContext, out: &mut InputOutcome) {
    let Some(p) = response.interact_pointer_pos() else {
        return;
    };
    if response.double_clicked() {
        double_click(p, cx, out);
    } else if response.clicked() {
        let shift = ctx.input(|i| i.modifiers.shift);
        out.commands
            .push(click_command(cx.plan.station_at(p), shift));
    }
}

/// Plain click selects only what was clicked (or clears on empty space).
/// Shift+click on a station adds it to, or removes it from, the selection.
fn click_command(hit: Option<StationId>, shift: bool) -> Command {
    match hit {
        Some(id) if shift => Command::Toggle(id),
        _ => Command::Select(hit),
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

/// Left-drag: move stations, or draw a box with Shift. Any other drag pans.
fn drags(ctx: &Context, response: &Response, cx: &InputContext, out: &mut InputOutcome) {
    if let Some(drag) = &cx.interaction.drag {
        continue_drag(drag, response, cx, out);
        return;
    }
    if let Some((from, _)) = cx.interaction.marquee {
        continue_marquee(from, response, cx, out);
        return;
    }
    if response.drag_started_by(PointerButton::Primary) && begin_drag(ctx, response, cx, out) {
        return;
    }
    if response.dragged() {
        let delta = response.drag_delta();
        out.commands.push(Command::Pan(Vec2::new(delta.x, delta.y)));
    }
}

/// Start a station drag or a box select at `origin`. Returns `false` when the
/// drag should pan instead.
fn begin_drag(
    ctx: &Context,
    response: &Response,
    cx: &InputContext,
    out: &mut InputOutcome,
) -> bool {
    let Some(origin) = ctx.input(|i| i.pointer.press_origin()) else {
        return false;
    };
    if let Some(hit) = cx.plan.station_at(origin) {
        begin_station_drag(hit, origin, cx, out);
        return true;
    }
    let shift = ctx.input(|i| i.modifiers.shift);
    if shift {
        let here = response.interact_pointer_pos().unwrap_or(origin);
        out.commands.push(Command::Marquee(Some((origin, here))));
    }
    shift
}

/// Dragging a selected station moves the whole selection. Dragging any other
/// station selects only it first.
fn begin_station_drag(hit: StationId, origin: Pos2, cx: &InputContext, out: &mut InputOutcome) {
    let ids: Vec<StationId> = if cx.interaction.is_selected(hit) {
        cx.interaction.selection.iter().copied().collect()
    } else {
        out.commands.push(Command::Select(Some(hit)));
        vec![hit]
    };
    let moving = ids
        .into_iter()
        .filter_map(|id| cx.map.stations.get(&id).map(|s| (id, s.position)))
        .collect();
    let grab = cx.plan.camera.view_to_world(to_vec2(origin));
    let drag = StationDrag {
        moving,
        grab,
        delta: Vec2::ZERO,
    };
    out.commands.push(Command::Drag(Some(drag)));
}

fn continue_drag(
    drag: &StationDrag,
    response: &Response,
    cx: &InputContext,
    out: &mut InputOutcome,
) {
    match response.interact_pointer_pos() {
        Some(p) if response.dragged() && !response.drag_stopped() => {
            let delta = cx.plan.camera.view_to_world(to_vec2(p)) - drag.grab;
            out.commands.push(Command::Drag(Some(StationDrag {
                delta,
                ..drag.clone()
            })));
        }
        _ => {
            out.edits.extend(drop_edit(drag));
            out.commands.push(Command::Drag(None));
        }
    }
}

/// On release each station snaps to the grid on its own. All the moves form
/// one batch, so a single undo puts them back.
fn drop_edit(drag: &StationDrag) -> Option<Edit> {
    let moves: Vec<Edit> = drag
        .moving
        .iter()
        .filter_map(|&(id, start)| {
            let to = snap_to_grid(start + drag.delta, GRID_CELL);
            (to != start).then_some(Edit::MoveStation { id, to })
        })
        .collect();
    (!moves.is_empty()).then_some(Edit::Batch(moves))
}

fn continue_marquee(from: Pos2, response: &Response, cx: &InputContext, out: &mut InputOutcome) {
    if response.drag_stopped() {
        let to = cx.interaction.marquee.map_or(from, |m| m.1);
        let rect = Rect::from_two_pos(from, to);
        out.commands
            .push(Command::AddToSelection(cx.plan.stations_in(rect)));
        out.commands.push(Command::Marquee(None));
    } else if let Some(p) = response.interact_pointer_pos() {
        out.commands.push(Command::Marquee(Some((from, p))));
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
