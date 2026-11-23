use super::{Command, InputContext, InputOutcome};
use eframe::egui::{Context, Event, Key, Modifiers};
use metromind_core::editing::Edit;
use metromind_core::energy::Touch;
use metromind_core::model::{LINE_COLOR_COUNT, Line, LineId, MetroMap, StationId};

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
        Key::Escape if cx.interaction.selected.is_some() => Command::Select(None),
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
        _ => return false,
    };
    out.commands.push(command);
    true
}

/// Shortcuts that act on the selected station.
fn selection_shortcut(key: Key, cx: &InputContext, out: &mut InputOutcome) -> bool {
    let Some(id) = cx.interaction.selected else {
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
        Key::Delete => {
            out.edits.push(Edit::DeleteStation { id });
            out.commands.push(Command::Select(None));
            out.commands.push(Command::Notice(
                "Idea removed. Ctrl+Z brings it back.".into(),
            ));
        }
        Key::R => {
            out.commands.push(Command::Touch(id, Touch::Rekindle));
            out.commands.push(Command::Notice(format!(
                "Rekindled “{}”",
                station.display_title()
            )));
        }
        Key::N => new_line(id, cx.map, out),
        _ => return false,
    }
    true
}

/// `1`–`9` adds the selection to that line; with Shift, removes it.
fn line_shortcut(index: usize, remove: bool, cx: &InputContext, out: &mut InputOutcome) -> bool {
    let Some(id) = cx.interaction.selected else {
        return false;
    };
    let number = index + 1;
    let Some(line) = cx.map.line_at(index) else {
        let hint = format!("There's no line {number} yet. Press N to start one from this idea.");
        out.commands.push(Command::Notice(hint));
        return true;
    };
    match (remove, line.index_of(id)) {
        (true, Some(index)) => out.edits.push(Edit::RemoveFromLine {
            line: line.id,
            index,
        }),
        (false, None) => out.edits.push(Edit::AddToLine {
            line: line.id,
            station: id,
            index: line.stations.len(),
        }),
        (true, None) => out.commands.push(Command::Notice(format!(
            "This idea isn't on {}.",
            line.name
        ))),
        (false, Some(_)) => out
            .commands
            .push(Command::Notice(format!("Already on {}.", line.name))),
    }
    true
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

/// Start renaming the selected station with whatever was typed.
fn start_typing(ctx: &Context, cx: &InputContext, out: &mut InputOutcome) {
    let Some(id) = cx.interaction.selected else {
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
