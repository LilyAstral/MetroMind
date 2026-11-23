//! The Ctrl+F search box: a field at the top of the map with up to eight matches
//! below it. ↑/↓ or the wheel move the highlight, Enter or a click jumps,
//! Esc closes.

use crate::render::colors::line_color;
use eframe::egui::{
    Align2, Area, Color32, Context, FontId, Frame, Id, Key, Order, Pos2, Rect, Response, Sense,
    TextEdit, Ui, vec2,
};
use metromind_core::editing::search;
use metromind_core::model::{MetroMap, StationId};

pub const MAX_RESULTS: usize = 8;
const WIDTH: f32 = 360.0;
const ROW_HEIGHT: f32 = 44.0;
const TOP_MARGIN: f32 = 12.0;
const SWATCH_RADIUS: f32 = 5.0;
const SWATCH_GAP: f32 = 14.0;
const HIGHLIGHT: Color32 = Color32::from_rgb(58, 65, 77);
const TEXT: Color32 = Color32::from_rgb(237, 230, 218);

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchState {
    pub query: String,
    pub highlight: usize,
    /// Focus the field on the first frame, so typing starts at once.
    fresh: bool,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            fresh: true,
            ..Self::default()
        }
    }
}

/// What the search box asks the app to do.
pub enum Action {
    Jump(StationId),
    Close,
}

/// Draw the box over the map `viewport`. Keys are read here, since the field
/// has focus and the map's shortcuts are suppressed while it does.
pub fn show(
    ctx: &Context,
    state: &mut SearchState,
    map: &MetroMap,
    viewport: Rect,
) -> Option<Action> {
    let results = search(map, &state.query, MAX_RESULTS);
    state.highlight = state.highlight.min(results.len().saturating_sub(1));
    if let Some(action) = keys(ctx, state, &results) {
        return Some(action);
    }
    let at = pos_inside(viewport);
    Area::new(Id::unique("search_box"))
        .order(Order::Foreground)
        .fixed_pos(at)
        .show(ctx, |ui| {
            Frame::popup(ui.style())
                .show(ui, |ui| panel(ui, state, map, &results))
                .inner
        })
        .inner
}

fn keys(ctx: &Context, state: &mut SearchState, results: &[StationId]) -> Option<Action> {
    let last = results.len().saturating_sub(1);
    ctx.input(|i| {
        if i.key_pressed(Key::Escape) {
            return Some(Action::Close);
        }
        if i.key_pressed(Key::ArrowDown) {
            state.highlight = (state.highlight + 1).min(last);
        }
        if i.key_pressed(Key::ArrowUp) {
            state.highlight = state.highlight.saturating_sub(1);
        }
        if i.key_pressed(Key::Enter) {
            return results.get(state.highlight).map(|&id| Action::Jump(id));
        }
        None
    })
}

fn panel(
    ui: &mut Ui,
    state: &mut SearchState,
    map: &MetroMap,
    results: &[StationId],
) -> Option<Action> {
    ui.set_width(WIDTH);
    let field = ui.add(
        TextEdit::singleline(&mut state.query)
            .hint_text("Find an idea")
            .desired_width(WIDTH)
            .min_size(vec2(WIDTH, ROW_HEIGHT)),
    );
    if std::mem::take(&mut state.fresh) {
        field.request_focus();
    }
    if state.query.trim().is_empty() {
        return None;
    }
    if results.is_empty() {
        ui.label("Nothing matches yet");
        return None;
    }
    let mut action = None;
    for (index, &id) in results.iter().enumerate() {
        let response = row(ui, map, id, index == state.highlight);
        if response.hovered() {
            state.highlight = index;
        }
        if response.clicked() {
            action = Some(Action::Jump(id));
        }
    }
    action
}

fn row(ui: &mut Ui, map: &MetroMap, id: StationId, highlighted: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(WIDTH, ROW_HEIGHT), Sense::click());
    let painter = ui.painter();
    if highlighted {
        painter.rect_filled(rect, 6.0, HIGHLIGHT);
    }
    let title = map
        .stations
        .get(&id)
        .map_or("untitled idea", |s| s.display_title());
    let text_at = rect.left_center() + vec2(10.0, 0.0);
    painter.text(
        text_at,
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(14.0),
        TEXT,
    );
    let mut x = rect.right() - 12.0;
    for line in map.ordered_lines().filter(|l| l.contains(id)) {
        painter.circle_filled(
            Pos2::new(x, rect.center().y),
            SWATCH_RADIUS,
            line_color(map, line),
        );
        x -= SWATCH_GAP;
    }
    response
}

/// Top centre of the map, a fixed distance down.
fn pos_inside(viewport: Rect) -> Pos2 {
    Pos2::new(
        viewport.center().x - WIDTH / 2.0,
        viewport.top() + TOP_MARGIN,
    )
}
