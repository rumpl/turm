use std::{
    ffi::CStr,
    os::fd::{AsRawFd, OwnedFd},
};

use ansi::{Ansi, AnsiOutput, SelectGraphicRendition};
use eframe::egui;
use egui::{Color32, Event, FontFamily, FontId, InputState, Key, Rect, TextStyle};

mod ansi;
mod ansi_codes;

fn set_nonblock(fd: &OwnedFd) {
    let flags = nix::fcntl::fcntl(fd.as_raw_fd(), nix::fcntl::FcntlArg::F_GETFL).unwrap();
    let mut flags =
        nix::fcntl::OFlag::from_bits(flags & nix::fcntl::OFlag::O_ACCMODE.bits()).unwrap();
    flags.set(nix::fcntl::OFlag::O_NONBLOCK, true);

    nix::fcntl::fcntl(fd.as_raw_fd(), nix::fcntl::FcntlArg::F_SETFL(flags)).unwrap();
}

fn main() {
    let options = eframe::NativeOptions::default();

    let fd = unsafe {
        let result = nix::pty::forkpty(None, None).unwrap();
        match result.fork_result {
            nix::unistd::ForkResult::Parent { child: _ } => {
                // TODO: wait for the child to exit and then exit Turm
            }
            nix::unistd::ForkResult::Child => {
                let command = CStr::from_bytes_with_nul(b"/bin/sh\0").unwrap();
                let args = [command];
                let _ = nix::unistd::execv(command, &args);
            }
        }
        result.master
    };

    set_nonblock(&fd);

    _ = eframe::run_native(
        "💩 Turm 💩",
        options,
        Box::new(|cc| Box::<Turm>::new(Turm::new(cc, fd))),
    );
}

struct Turm {
    buf: Vec<AnsiOutput>,
    fd: OwnedFd,
    ansi: Ansi,
}

impl Turm {
    fn new(cc: &eframe::CreationContext<'_>, fd: OwnedFd) -> Self {
        cc.egui_ctx.style_mut(|style| {
            style.override_text_style = Some(TextStyle::Monospace);
            for (_text_style, font_id) in style.text_styles.iter_mut() {
                font_id.size = 24.0
            }
        });
        Self {
            fd,
            buf: vec![],
            ansi: Ansi::new(),
        }
    }
}

fn write_input_to_terminal(input: &InputState, fd: &OwnedFd) {
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
            } => Some("\n"),
            _ => None,
        };

        if let Some(text) = text {
            let _ret = nix::unistd::write(fd.as_raw_fd(), text.as_bytes());
        }
    }
}

impl eframe::App for Turm {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut buf = vec![0u8; 4096];
        let ret = nix::unistd::read(self.fd.as_raw_fd(), &mut buf);

        if let Ok(read_size) = ret {
            let mut ansi_res = self.ansi.push(&buf[0..read_size]);
            self.buf.append(&mut ansi_res);
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.input(|input_state| {
                write_input_to_terminal(input_state, &self.fd);
            });

            let font_id = FontId {
                size: 24.0,
                family: FontFamily::Monospace,
            };

            let mut job = egui::text::LayoutJob::default();

            let mut color: Color32 = Color32::WHITE;
            let mut chars = 0;
            for out in &self.buf {
                match out {
                    AnsiOutput::Text(str) => {
                        let text = String::from_utf8_lossy(&str);
                        chars += text.len();
                        job.append(
                            &text,
                            0.0,
                            egui::text::TextFormat {
                                font_id: font_id.clone(),
                                color,
                                ..Default::default()
                            },
                        );
                    }
                    AnsiOutput::SGR(c) => {
                        color = (*c).into();
                    }
                }
            }

            ui.label(job);

            let mut width = 0.0;
            ctx.fonts(|font| {
                width = font.glyph_width(&font_id, 'i');
            });

            // This is how to paint a cursor
            let painter = ui.painter();
            let pos = egui::pos2(width * (chars + 1) as f32 - 5.0, 8.0);
            let size = egui::vec2(10.0, 25.0);
            painter.rect_filled(Rect::from_min_size(pos, size), 0.0, Color32::GRAY);
        });
    }
}

impl From<SelectGraphicRendition> for Color32 {
    fn from(sgr: SelectGraphicRendition) -> Self {
        match sgr {
            SelectGraphicRendition::ForegroundBlack => Color32::BLACK,
            SelectGraphicRendition::ForegroundRed => Color32::RED,
            SelectGraphicRendition::ForegroundGreen => Color32::GREEN,
            SelectGraphicRendition::ForegroundYellow => Color32::YELLOW,
            _ => Color32::WHITE,
        }
    }
}
