use super::{Command, InputContext, InputOutcome};
use eframe::egui::{Context, Event, Key, Modifiers};
use metromind_core::editing::Edit;
use metromind_core::energy::Touch;
use metromind_core::model::{LINE_COLOR_COUNT, Line, LineId, MetroMap, StationId};
use std::collections::BTreeSet;

const DIGIT_KEYS: [Key; 9] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
];

pub fn collect(ctx: &Context, cx: &InputContext, out: &mut InputOutcome) {
    let fired = ctx.input(|i| {
        let mut fired = false;
        for event in &i.events {
            if let Event::Key {
                key,
                physical_key,
                pressed: true,
                modifiers,
                ..
            } = event
            {
                fired |= shortcut(*key, physical_key.unwrap_or(*key), *modifiers, cx, out);
            }
        }
        fired
    });
    if !fired {
        start_typing(ctx, cx, out);
    }
    out.pin_lens = ctx.input(|i| i.key_down(Key::Space));
}

/// Returns whether the key was a shortcut, so its text isn't also typed.
/// Digits use the physical key so Shift+1 is still "1" on every layout.
fn shortcut(
    key: Key,
    physical: Key,
    mods: Modifiers,
    cx: &InputContext,
    out: &mut InputOutcome,
) -> bool {
    if mods.command {
        return command_shortcut(key, mods.shift, out);
    }
    if let Some(n) = DIGIT_KEYS.iter().position(|&k| k == physical) {
        return line_shortcut(n, mods.shift, cx, out);
    }
    if mods.any() {
        return false;
    }
    let command = match key {
        Key::L => Command::ToggleLens,
        Key::Home => Command::FitMap,
        Key::F1 => Command::ToggleHelp,
        Key::Escape if !cx.interaction.selection.is_empty() => Command::Select(None),
        _ => return selection_shortcut(key, cx, out),
    };
    out.commands.push(command);
    true
}

fn command_shortcut(key: Key, shift: bool, out: &mut InputOutcome) -> bool {
    let command = match key {
        Key::Z if shift => Command::Redo,
        Key::Z => Command::Undo,
        Key::Y => Command::Redo,
        Key::S if shift => Command::SaveAs,
        Key::S => Command::Save,
        Key::O => Command::Open,
        Key::N => Command::NewMap,
        Key::A => Command::SelectAll,
        _ => return false,
    };
    out.commands.push(command);
    true
}

/// Shortcuts that act on the whole selection: Del and R.
/// F2, Enter and N act on the primary station only.
fn selection_shortcut(key: Key, cx: &InputContext, out: &mut InputOutcome) -> bool {
    if cx.interaction.selection.is_empty() {
        return false;
    }
    match key {
        Key::Delete => delete_selection(cx, out),
        Key::R => rekindle_selection(cx, out),
        _ => return primary_shortcut(key, cx, out),
    }
    true
}

fn primary_shortcut(key: Key, cx: &InputContext, out: &mut InputOutcome) -> bool {
    let Some(id) = cx.interaction.primary else {
        return false;
    };
    let Some(station) = cx.map.stations.get(&id) else {
        return false;
    };
    match key {
        Key::F2 => out.commands.push(Command::StartRename {
            id,
            text: station.title.clone(),
        }),
        Key::Enter => out.commands.push(Command::OpenNotes(id)),
        Key::N => new_line(id, cx.map, out),
        _ => return false,
    }
    true
}

fn delete_selection(cx: &InputContext, out: &mut InputOutcome) {
    let edits = cx
        .interaction
        .selection
        .iter()
        .map(|&id| Edit::DeleteStation { id })
        .collect();
    out.edits.push(Edit::Batch(edits));
    out.commands.push(Command::Select(None));
    let count = cx.interaction.selection.len();
    let notice = match count {
        1 => "Idea removed. Ctrl+Z brings it back.".to_owned(),
        _ => format!("{count} ideas removed. Ctrl+Z brings them back."),
    };
    out.commands.push(Command::Notice(notice));
}

fn rekindle_selection(cx: &InputContext, out: &mut InputOutcome) {
    for &id in &cx.interaction.selection {
        out.commands.push(Command::Touch(id, Touch::Rekindle));
    }
    let notice = match cx
        .interaction
        .primary
        .and_then(|id| cx.map.stations.get(&id))
    {
        Some(station) if cx.interaction.selection.len() == 1 => {
            format!("Rekindled “{}”", station.display_title())
        }
        _ => format!("Rekindled {} ideas", cx.interaction.selection.len()),
    };
    out.commands.push(Command::Notice(notice));
}

/// `1`–`9` adds the selection to that line; with Shift, removes it.
fn line_shortcut(index: usize, remove: bool, cx: &InputContext, out: &mut InputOutcome) -> bool {
    if cx.interaction.selection.is_empty() {
        return false;
    }
    let number = index + 1;
    let Some(line) = cx.map.line_at(index) else {
        let hint = format!("There's no line {number} yet. Press N to start one from this idea.");
        out.commands.push(Command::Notice(hint));
        return true;
    };
    let edits = line_edits(line, remove, &cx.interaction.selection);
    if !edits.is_empty() {
        out.edits.push(Edit::Batch(edits));
    } else {
        out.commands
            .push(Command::Notice(skipped_notice(line, remove, cx)));
    }
    true
}

/// Edits that add each selected station to `line` (or take it off, with
/// `remove`). Stations where the action does nothing are skipped. Removals go
/// from the highest index down so the earlier ones stay valid.
fn line_edits(line: &Line, remove: bool, selection: &BTreeSet<StationId>) -> Vec<Edit> {
    if remove {
        let mut indexes: Vec<usize> = selection
            .iter()
            .filter_map(|&id| line.index_of(id))
            .collect();
        indexes.sort_unstable_by(|a, b| b.cmp(a));
        return indexes
            .into_iter()
            .map(|index| Edit::RemoveFromLine {
                line: line.id,
                index,
            })
            .collect();
    }
    let mut next = line.stations.len();
    selection
        .iter()
        .filter(|&&id| !line.contains(id))
        .map(|&station| {
            let edit = Edit::AddToLine {
                line: line.id,
                station,
                index: next,
            };
            next += 1;
            edit
        })
        .collect()
}

fn skipped_notice(line: &Line, remove: bool, cx: &InputContext) -> String {
    let single = cx.interaction.selection.len() == 1;
    match (remove, single) {
        (true, true) => format!("This idea isn't on {}.", line.name),
        (true, false) => format!("None of the selected ideas are on {}.", line.name),
        (false, true) => format!("Already on {}.", line.name),
        (false, false) => format!("All selected ideas are already on {}.", line.name),
    }
}

fn new_line(station: StationId, map: &MetroMap, out: &mut InputOutcome) {
    if map.lines.len() >= usize::from(LINE_COLOR_COUNT) {
        let hint = "All 9 lines are in use. Add this idea to one with 1–9.";
        out.commands.push(Command::Notice(hint.into()));
        return;
    }
    let line = Line {
        id: LineId(map.peek_id()),
        name: format!("Line {}", map.lines.len() + 1),
        color_index: map.free_color_index().unwrap_or(0),
        stations: vec![station],
        custom_color: None,
    };
    let hint = format!("Started {}. Rename it in the side panel.", line.name);
    out.edits.push(Edit::CreateLine {
        line,
        position: map.line_order.len(),
    });
    out.commands.push(Command::Notice(hint));
}

/// Start renaming the primary station with whatever was typed.
fn start_typing(ctx: &Context, cx: &InputContext, out: &mut InputOutcome) {
    let Some(id) = cx.interaction.primary else {
        return;
    };
    let typed = ctx.input(|i| {
        i.events.iter().find_map(|e| match e {
            Event::Text(t) if !t.trim().is_empty() && !i.modifiers.command => Some(t.clone()),
            _ => None,
        })
    });
    if let Some(text) = typed {
        out.commands.push(Command::StartRename { id, text });
    }
}
