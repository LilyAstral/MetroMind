//! Keys and wheel while the line picker is open. The rest of the map ignores
//! input until the picker closes.

use super::{Command, InputContext, InputOutcome};
use crate::ui::picker::{Choice, Picker, rows};
use eframe::egui::{Context, Event, Key};

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

pub fn collect(ctx: &Context, cx: &InputContext, picker: &Picker, out: &mut InputOutcome) {
    let rows = rows(cx.map, picker);
    let lines = rows
        .iter()
        .filter(|r| matches!(r.choice, Choice::Line(_)))
        .count();
    let new_row = rows.len() - 1;
    ctx.input(|i| {
        for event in &i.events {
            let command = match event {
                Event::Key {
                    key,
                    physical_key,
                    pressed: true,
                    ..
                } => {
                    let physical = physical_key.unwrap_or(*key);
                    key_command(*key, physical, picker.highlight, lines, new_row)
                }
                Event::MouseWheel { delta, .. } if delta.y != 0.0 => {
                    Some(Command::PickerStep(-(delta.y.signum() as i32)))
                }
                _ => None,
            };
            out.commands.extend(command);
        }
    });
}

/// Enter picks the highlight, `N` picks "New line", `1`–`9` pick a line by
/// its number, and `Esc` cancels.
fn key_command(
    key: Key,
    physical: Key,
    highlight: usize,
    lines: usize,
    new_row: usize,
) -> Option<Command> {
    match key {
        Key::Escape => Some(Command::PickerCancel),
        Key::Enter => Some(Command::PickerPick(highlight)),
        Key::N => Some(Command::PickerPick(new_row)),
        _ => {
            let number = DIGIT_KEYS.iter().position(|&k| k == physical)?;
            Some(if number < lines {
                Command::PickerPick(number)
            } else {
                Command::Notice(format!("There's no line {} yet.", number + 1))
            })
        }
    }
}
