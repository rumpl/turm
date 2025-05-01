use std::{
    os::fd::{AsRawFd, OwnedFd},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use egui::{Event, InputState, Key, Modifiers};

use crate::turm::Turm;

pub enum TerminalGuiInputMessage {
    Text(Vec<u8>),
    ScrollUp(u32),
    ScrollDown(u32),
}

/// TerminalInput processes the input from the GUI and sends it back to the
/// child terminal.
pub struct TerminalGuiInput {
    write_fd: OwnedFd,
    tx: Sender<TerminalGuiInputMessage>,
}

impl Clone for TerminalGuiInput {
    fn clone(&self) -> Self {
        Self {
            write_fd: self.write_fd.try_clone().unwrap(),
            tx: self.tx.clone(),
        }
    }
}

impl TerminalGuiInput {
    pub fn new(turm: Arc<Mutex<Turm>>, write_fd: OwnedFd) -> Self {
        let (tx, rx) = mpsc::channel::<TerminalGuiInputMessage>();
        let terminal_input = Self { write_fd, tx };

        // Start the input handling thread immediately
        Self::start_input_thread(Arc::clone(&turm), terminal_input.write_fd.as_raw_fd(), rx);

        terminal_input
    }

    fn start_input_thread(
        turm: Arc<Mutex<Turm>>,
        write_fd_raw: i32,
        rx: Receiver<TerminalGuiInputMessage>,
    ) {
        thread::spawn(move || loop {
            if let Ok(input) = rx.recv() {
                match input {
                    TerminalGuiInputMessage::Text(text) => {
                        let _ = nix::unistd::write(write_fd_raw, &text);
                    }
                    TerminalGuiInputMessage::ScrollUp(delta) => {
                        let mut t = turm.lock().unwrap();
                        t.scroll_up(delta, false);
                    }
                    TerminalGuiInputMessage::ScrollDown(delta) => {
                        let mut t = turm.lock().unwrap();
                        t.scroll_down(delta, false);
                    }
                }
            }
        });
    }

    pub fn write_input_to_terminal(&self, input: &InputState) {
        for event in &input.events {
            let text = match event {
                Event::Text(text) => Some(text.as_str()),
                Event::Key {
                    key: Key::Backspace,
                    pressed: true,
                    ..
                } => Some("\x7F"),
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
                    let _ = self.tx.send(TerminalGuiInputMessage::Text(vec![m]));
                    None
                }
                Event::MouseWheel {
                    unit: _,
                    delta,
                    modifiers: _,
                } => {
                    if delta.y > 0.0 {
                        let _ = self
                            .tx
                            .send(TerminalGuiInputMessage::ScrollDown(delta.y as u32));
                    } else {
                        let _ = self
                            .tx
                            .send(TerminalGuiInputMessage::ScrollUp(delta.y.abs() as u32));
                    }
                    None
                }
                _ => None,
            };

            if let Some(text) = text {
                let _ = self.tx.send(TerminalGuiInputMessage::Text(text.into()));
            }
        }
    }
}
