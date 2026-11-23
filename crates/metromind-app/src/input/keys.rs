use super::{Action, Command, InputContext, InputOutcome};
use eframe::egui::{Context, Event, Key, Modifiers};

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
        Key::Escape if cx.interaction.highlight.is_some() => Command::HighlightLine(None),
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
        Key::F => Command::OpenSearch,
        _ => return false,
    };
    out.commands.push(command);
    true
}

/// Del and R act on the whole selection. F2, Enter and N act on the primary.
fn selection_shortcut(key: Key, cx: &InputContext, out: &mut InputOutcome) -> bool {
    if cx.interaction.selection.is_empty() {
        return false;
    }
    match key {
        Key::Delete => out.commands.push(Command::Act(Action::Delete)),
        Key::R => {
            out.commands.push(Command::Act(Action::Rekindle));
        }
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
        Key::N => out.commands.push(Command::Act(Action::NewLine(id))),
        _ => return false,
    }
    true
}

/// `1`–`9` adds the selection to that line; with Shift, removes it.
fn line_shortcut(index: usize, remove: bool, cx: &InputContext, out: &mut InputOutcome) -> bool {
    if cx.interaction.selection.is_empty() {
        return false;
    }
    match cx.map.line_at(index) {
        Some(line) => out.commands.push(Command::Act(Action::ToLine {
            line: line.id,
            remove,
        })),
        None => {
            let hint = format!(
                "There's no line {} yet. Press N to start one from this idea.",
                index + 1
            );
            out.commands.push(Command::Notice(hint));
        }
    }
    true
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
