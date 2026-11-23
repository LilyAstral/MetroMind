//! egui input → [`Edit`]s (model changes) and [`Command`]s (view changes).
//!
//! Input only reads the map and last frame's plan; the app applies the result.

mod keys;
mod picker;
mod pointer;

use crate::frame_plan::FramePlan;
use crate::ui::picker::Picker;
use eframe::egui::{Context, Pos2, Response};
use metromind_core::Vec2;
use metromind_core::chrono::{DateTime, Utc};
use metromind_core::editing::Edit;
use metromind_core::energy::{EnergySettings, Touch};
use metromind_core::model::{MetroMap, StationId};
use std::collections::BTreeSet;

/// View state that input needs to interpret gestures.
#[derive(Debug, Default, Clone)]
pub struct Interaction {
    /// Every selected station.
    pub selection: BTreeSet<StationId>,
    /// The station clicked most recently. Always in `selection` when set.
    pub primary: Option<StationId>,
    pub hovered: Option<StationId>,
    pub drag: Option<StationDrag>,
    /// A right-drag from a station that is connecting or branching.
    pub connect: Option<ConnectDrag>,
    /// The line picker, open after a connection drag is released.
    pub picker: Option<Picker>,
    /// A Shift+drag selection box: where it started and where the pointer is.
    pub marquee: Option<(Pos2, Pos2)>,
    /// A text box (rename, notes, line name) owns the keyboard.
    pub typing: bool,
}

impl Interaction {
    pub fn is_selected(&self, id: StationId) -> bool {
        self.selection.contains(&id)
    }
}

/// A connection drag in progress.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectDrag {
    pub from: StationId,
    pub pointer: Pos2,
}

/// Stations being dragged together. The model only changes on release.
#[derive(Debug, Clone, PartialEq)]
pub struct StationDrag {
    /// Each moving station with the position it had when the drag began.
    pub moving: Vec<(StationId, Vec2)>,
    /// World position under the pointer when the drag began.
    pub grab: Vec2,
    /// Pointer movement since the drag began, in world units, not snapped.
    pub delta: Vec2,
}

/// View-level changes. Applied by the app after this frame's edits.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Replace the selection with this one station (`None` clears it).
    Select(Option<StationId>),
    /// Shift+click: add the station to the selection, or remove it.
    Toggle(StationId),
    /// Box select: add these stations to the selection.
    AddToSelection(Vec<StationId>),
    SelectAll,
    /// Show the Shift+drag selection box, or hide it with `None`.
    Marquee(Option<(Pos2, Pos2)>),
    Touch(StationId, Touch),
    Drag(Option<StationDrag>),
    Connect(Option<ConnectDrag>),
    OpenPicker(Picker),
    PickerStep(i32),
    PickerPick(usize),
    PickerCancel,
    Pan(Vec2),
    Zoom {
        anchor: Vec2,
        factor: f32,
    },
    AdjustLensStrength(f32),
    AdjustLensRadius(f32),
    ToggleLens,
    FitMap,
    StartRename {
        id: StationId,
        text: String,
    },
    OpenNotes(StationId),
    Undo,
    Redo,
    NewMap,
    Open,
    Save,
    SaveAs,
    LoadSample,
    ToggleHelp,
    Notice(String),
}

#[derive(Debug, Default)]
pub struct InputOutcome {
    pub edits: Vec<Edit>,
    pub commands: Vec<Command>,
    pub hovered: Option<StationId>,
    /// Space is held: the lens stays where it is.
    pub pin_lens: bool,
}

/// Everything input reads, bundled to keep signatures short.
pub struct InputContext<'a> {
    pub plan: &'a FramePlan,
    pub map: &'a MetroMap,
    pub interaction: Interaction,
    pub now: DateTime<Utc>,
    pub energy: &'a EnergySettings,
}

/// Read this frame's input on the map area. Keyboard shortcuts are skipped
/// while any text box has focus, so typing never triggers them.
pub fn collect(ctx: &Context, map_response: &Response, cx: &InputContext) -> InputOutcome {
    let mut out = InputOutcome::default();
    if let Some(picker) = &cx.interaction.picker {
        picker::collect(ctx, cx, picker, &mut out);
        return out;
    }
    pointer::collect(ctx, map_response, cx, &mut out);
    if !cx.interaction.typing && !ctx.egui_wants_keyboard_input() {
        keys::collect(ctx, cx, &mut out);
    }
    out
}
