//! Keyboard routing while a text is edited on the canvas: typed text,
//! editing keys and clipboard events go to the session instead of commands.

use egui::{Event, ImeEvent, Key, Modifiers};
use tp_text::Motion;

use crate::workspace::Workspace;

/// Word motions use Alt on macOS and Ctrl elsewhere; line motions use Cmd
/// on macOS.
fn word_modifier(m: Modifiers, is_mac: bool) -> bool {
    if is_mac { m.alt } else { m.ctrl }
}

/// Consumes this frame's text events for the session of `ws`.
pub fn handle(ctx: &egui::Context, ws: &mut Workspace, now: f64) {
    if !ws.is_editing_text() {
        return;
    }
    let is_mac = ctx.os().is_mac();
    let events = ctx.input_mut(|i| {
        let mut taken = Vec::new();
        i.events.retain(|e| {
            let ours = match e {
                Event::Text(_) | Event::Paste(_) | Event::Copy | Event::Cut => true,
                Event::Ime(ImeEvent::Commit(_)) => true,
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => is_editing_key(*key, *modifiers),
                _ => false,
            };
            if ours {
                taken.push(e.clone());
            }
            !ours
        });
        taken
    });
    for event in events {
        match event {
            Event::Text(text) | Event::Ime(ImeEvent::Commit(text)) => {
                let text: String = text.chars().filter(|c| !c.is_control()).collect();
                if !text.is_empty() {
                    ws.text_insert(&text, now);
                }
            }
            Event::Paste(text) => ws.text_insert(&text, now),
            Event::Copy => {
                if let Some(session) = &ws.text_session
                    && session.edit.has_selection()
                {
                    ctx.copy_text(session.edit.selected_text().to_owned());
                }
            }
            Event::Cut => {
                let mut cut = String::new();
                ws.with_session(now, |edit, _| cut = edit.cut());
                if !cut.is_empty() {
                    ctx.copy_text(cut);
                }
            }
            Event::Key { key, modifiers, .. } => key_press(ws, key, modifiers, is_mac, now),
            _ => {}
        }
    }
}

fn is_editing_key(key: Key, m: Modifiers) -> bool {
    match key {
        Key::Escape
        | Key::Enter
        | Key::Backspace
        | Key::Delete
        | Key::ArrowLeft
        | Key::ArrowRight
        | Key::ArrowUp
        | Key::ArrowDown
        | Key::Home
        | Key::End
        | Key::Tab => true,
        Key::A | Key::Z | Key::Y => m.command,
        _ => false,
    }
}

fn key_press(ws: &mut Workspace, key: Key, m: Modifiers, is_mac: bool, now: f64) {
    let extend = m.shift;
    let word = word_modifier(m, is_mac);
    let motion = match key {
        Key::ArrowLeft if m.command && is_mac => Some(Motion::Home),
        Key::ArrowRight if m.command && is_mac => Some(Motion::End),
        Key::ArrowLeft if word => Some(Motion::WordLeft),
        Key::ArrowRight if word => Some(Motion::WordRight),
        Key::ArrowLeft => Some(Motion::Left),
        Key::ArrowRight => Some(Motion::Right),
        Key::ArrowUp if m.command => Some(Motion::Start),
        Key::ArrowDown if m.command => Some(Motion::Finish),
        Key::ArrowUp => Some(Motion::Up),
        Key::ArrowDown => Some(Motion::Down),
        Key::Home if m.command => Some(Motion::Start),
        Key::End if m.command => Some(Motion::Finish),
        Key::Home => Some(Motion::Home),
        Key::End => Some(Motion::End),
        _ => None,
    };
    if let Some(motion) = motion {
        ws.text_move(motion, extend, now);
        return;
    }
    match key {
        Key::Escape => ws.end_text_session(now),
        Key::Enter => ws.text_insert("\n", now),
        Key::Backspace => ws.with_session(now, |edit, _| edit.backspace()),
        Key::Delete => ws.with_session(now, |edit, _| edit.delete()),
        Key::A => ws.with_session(now, |edit, _| edit.select_all()),
        Key::Z if m.shift => {
            ws.text_redo(now);
        }
        Key::Z => {
            ws.text_undo(now);
        }
        Key::Y => {
            ws.text_redo(now);
        }
        _ => {}
    }
}
