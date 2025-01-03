use std::sync::mpsc::Sender;

use egui::{Event, InputState, Key, Modifiers};

pub enum TerminalGuiInputMessage {
    Text(Vec<u8>),
    ScrollUp(u32),
    ScrollDown(u32),
}

/// TerminalInput processes the input from the GUI and sends it back to the
/// child terminal.
pub struct TerminalGuiInput {
    rtx: Sender<TerminalGuiInputMessage>,
}

impl TerminalGuiInput {
    pub fn new(rtx: Sender<TerminalGuiInputMessage>) -> Self {
        Self { rtx }
    }

    pub fn write_input_to_terminal(&self, input: &InputState) {
        for event in &input.events {
            let text = match event {
                Event::Text(text) => Some(text.as_str()),
                Event::Key {
                    key: Key::Backspace,
                    pressed: true,
                    ..
                } => Some("\u{8}"),
                Event::Key {
                    key: Key::Enter,
                    pressed: true,
                    ..
                } => Some("\r"),
                Event::Key {
                    key: Key::ArrowUp,
                    pressed: true,
                    ..
                } => Some("\x1bOA"),
                Event::Key {
                    key: Key::ArrowDown,
                    pressed: true,
                    ..
                } => Some("\x1bOB"),
                Event::Key {
                    key: Key::ArrowRight,
                    pressed: true,
                    ..
                } => Some("\x1bOC"),
                Event::Key {
                    key: Key::ArrowLeft,
                    pressed: true,
                    ..
                } => Some("\x1bOD"),
                Event::Key {
                    key: Key::Tab,
                    pressed: true,
                    ..
                } => Some("\t"),
                Event::Key {
                    key: Key::Escape,
                    pressed: true,
                    ..
                } => Some("\x1b"),
                Event::Key {
                    key,
                    modifiers: Modifiers { ctrl: true, .. },
                    pressed: true,
                    ..
                } => {
                    // Meh...
                    let n = key.name().chars().next().unwrap();
                    let mut m = n as u8;
                    m &= 0b1001_1111;
                    let _ = self.rtx.send(TerminalGuiInputMessage::Text(vec![m]));
                    None
                }
                Event::Scroll(vec) => {
                    if vec.y > 0.0 {
                        let _ = self
                            .rtx
                            .send(TerminalGuiInputMessage::ScrollDown(vec.y as u32));
                    } else {
                        let _ = self
                            .rtx
                            .send(TerminalGuiInputMessage::ScrollUp(vec.y.abs() as u32));
                    }
                    None
                }
                _ => None,
            };

            if let Some(text) = text {
                let _ = self.rtx.send(TerminalGuiInputMessage::Text(text.into()));
            }
        }
    }
}
