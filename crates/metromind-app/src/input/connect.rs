//! Right-drag from a station: connect it to another station, or branch off
//! empty space. Releasing opens the line picker.

use super::{Command, ConnectDrag, InputContext, InputOutcome};
use crate::frame_plan::to_vec2;
use crate::ui::picker::{Picker, Target};
use eframe::egui::{Context, PointerButton, Pos2, Response};
use metromind_core::geometry::{GRID_CELL, snap_to_grid};
use metromind_core::model::StationId;

/// A right-drag that starts on a station begins a connection. On empty space
/// it pans, so this returns `false` there.
pub(super) fn begin_connect(ctx: &Context, cx: &InputContext, out: &mut InputOutcome) -> bool {
    let Some(origin) = ctx.input(|i| i.pointer.press_origin()) else {
        return false;
    };
    let Some(from) = cx.plan.station_at(origin) else {
        return false;
    };
    let connect = ConnectDrag {
        from,
        pointer: origin,
    };
    out.commands.push(Command::Connect(Some(connect)));
    true
}

pub(super) fn continue_connect(
    connect: &ConnectDrag,
    response: &Response,
    cx: &InputContext,
    out: &mut InputOutcome,
) {
    let pointer = response.interact_pointer_pos().unwrap_or(connect.pointer);
    if response.drag_stopped_by(PointerButton::Secondary) {
        finish_connect(connect.from, pointer, cx, out);
    } else {
        let moved = ConnectDrag {
            pointer,
            ..connect.clone()
        };
        out.commands.push(Command::Connect(Some(moved)));
    }
}

/// Released on another station: pick a line to connect to. Released on empty
/// space: pick a line to branch onto, at the grid-snapped spot (or connect to
/// the station already there). Released on the origin: nothing happens.
fn finish_connect(from: StationId, pointer: Pos2, cx: &InputContext, out: &mut InputOutcome) {
    out.commands.push(Command::Connect(None));
    let target = match cx.plan.station_at(pointer) {
        Some(hit) => Target::Station(hit),
        None => {
            let spot = snap_to_grid(cx.plan.camera.view_to_world(to_vec2(pointer)), GRID_CELL);
            cx.map
                .stations
                .values()
                .find(|s| s.position == spot)
                .map_or(Target::Spot(spot), |s| Target::Station(s.id))
        }
    };
    if target == Target::Station(from) {
        return;
    }
    out.commands.push(Command::OpenPicker(Picker {
        origin: from,
        target,
        anchor: pointer,
        highlight: 0,
    }));
}
