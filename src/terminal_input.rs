use std::sync::mpsc::Sender;

use egui::{Event, InputState, Key, Modifiers};

pub struct TerminalInput {
    rtx: Sender<Vec<u8>>,
}

impl TerminalInput {
    pub fn new(rtx: Sender<Vec<u8>>) -> Self {
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
                    let _ = self.rtx.send(vec![m]);
                    None
                }
                _ => None,
            };

            if let Some(text) = text {
                let _ = self.rtx.send(text.into());
            }
        }
    }
}
