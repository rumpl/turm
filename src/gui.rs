use std::{
    os::fd::{AsRawFd, OwnedFd},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::{
    ansi::{Ansi, AnsiOutput, SelectGraphicRendition},
    turm::Turm,
};
use egui::{Color32, Event, FontFamily, FontId, InputState, Key, Modifiers, Rect, TextStyle};
use nix::ioctl_write_ptr_bad;

ioctl_write_ptr_bad!(
    set_window_size_ioctl,
    nix::libc::TIOCSWINSZ,
    nix::pty::Winsize
);

pub struct TurmGui {
    buf: Vec<AnsiOutput>,
    turm: Turm,
    ansi: Ansi,
    rx: Receiver<Vec<u8>>,
    rtx: Sender<Vec<u8>>,
}

impl TurmGui {
    pub fn new(cc: &eframe::CreationContext<'_>, fd: OwnedFd) -> Self {
        cc.egui_ctx.style_mut(|style| {
            style.override_text_style = Some(TextStyle::Monospace);
            for (_text_style, font_id) in style.text_styles.iter_mut() {
                font_id.size = 24.0;
            }
        });

        let cols: usize = 90;
        let rows: usize = 30;

        let ws = nix::pty::Winsize {
            ws_col: cols as u16,
            ws_row: rows as u16,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        unsafe {
            let _ = set_window_size_ioctl(fd.as_raw_fd(), &ws);
        }

        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let rs = cc.egui_ctx.clone();
        let fd2 = fd.try_clone().unwrap();
        // Thread that reads output from the shell and sends it to the gui
        thread::spawn(move || loop {
            let mut buf = vec![0u8; 4096];
            let ret = nix::unistd::read(fd.as_raw_fd(), &mut buf);
            if let Ok(s) = ret {
                if s != 0 {
                    let _ = tx.send(buf[0..s].to_vec());
                    rs.request_repaint();
                }
            } else {
                rs.request_repaint();
            }
        });

        let (rtx, rrx) = mpsc::channel::<Vec<u8>>();
        // Thread that gets user input and sends it to the shell
        thread::spawn(move || loop {
            let input = rrx.recv().unwrap();
            let _ret = nix::unistd::write(fd2.as_raw_fd(), &input);
        });

        Self {
            rx,
            rtx,
            buf: vec![],
            ansi: Ansi::new(),
            // TODO: calculate the right initial number of rows and columns
            turm: Turm::new(cols, rows),
        }
    }

    fn write_input_to_terminal(&self, input: &InputState) {
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

impl eframe::App for TurmGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font_size = 14.0;
        let line_height = font_size + 6.0;

        let ret = self.rx.try_recv();
        if let Ok(buf) = ret {
            let mut ansi_res = self.ansi.push(&buf);
            for q in &ansi_res {
                match q {
                    AnsiOutput::Text(str) => {
                        for c in str {
                            self.turm.input(*c);
                        }
                    }
                    AnsiOutput::ClearToEndOfLine(_mode) => self.turm.clear_to_end_of_line(),
                    AnsiOutput::ClearToEOS => self.turm.clear_to_eos(),
                    AnsiOutput::MoveCursor(x, y) => {
                        self.turm.move_cursor(*x, *y);
                    }
                    AnsiOutput::CursorUp(amount) => self
                        .turm
                        .move_cursor(self.turm.cursor.pos.x, self.turm.cursor.pos.y - amount),
                    AnsiOutput::CursorDown(amount) => self
                        .turm
                        .move_cursor(self.turm.cursor.pos.x, self.turm.cursor.pos.y + amount),
                    AnsiOutput::ScrollDown => self.turm.scroll_down(),
                    AnsiOutput::Backspace => self.turm.backspace(),
                    AnsiOutput::Sgr(c) => self.turm.color(*c),
                    AnsiOutput::Bell => println!("DING DONG"),
                }
            }
            self.buf.append(&mut ansi_res);
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.input(|input_state| {
                self.write_input_to_terminal(input_state);
            });

            let font_id = FontId {
                size: font_size,
                family: FontFamily::Monospace,
            };

            let mut job = egui::text::LayoutJob::default();
            for section in self.turm.grid.sections() {
                job.append(
                    &section.text,
                    0.0,
                    egui::text::TextFormat {
                        font_id: font_id.clone(),
                        color: section.fg.into(),
                        line_height: Some(line_height),
                        ..Default::default()
                    },
                );
            }
            let res = ui.label(job);

            let mut width = 0.0;
            ctx.fonts(|font| {
                width = font.glyph_width(&font_id, 'i');
            });

            let painter = ui.painter();
            let pos = egui::pos2(
                (self.turm.cursor.pos.x as f32) * width + res.rect.left(),
                (self.turm.cursor.pos.y as f32) * line_height + res.rect.top(),
            );
            let size = egui::vec2(width, font_size);
            painter.rect_filled(Rect::from_min_size(pos, size), 0.0, Color32::WHITE);
        });
    }
}

impl From<SelectGraphicRendition> for Color32 {
    fn from(sgr: SelectGraphicRendition) -> Self {
        match sgr {
            SelectGraphicRendition::Reset => Self::WHITE,
            SelectGraphicRendition::ForegroundBlack => Self::BLACK,
            SelectGraphicRendition::ForegroundRed => Self::RED,
            SelectGraphicRendition::ForegroundGreen => Self::GREEN,
            SelectGraphicRendition::ForegroundYellow => Self::YELLOW,
            SelectGraphicRendition::ForegroundBlue => Self::BLUE,
            SelectGraphicRendition::ForegroundMagenta => Self::from_rgb(255, 0, 255),
            SelectGraphicRendition::ForegroundCyan => Self::from_rgb(0, 255, 255),
            SelectGraphicRendition::ForegroundWhite => Self::WHITE,
            SelectGraphicRendition::ForegroundGrey => Self::GRAY, // lol
            SelectGraphicRendition::ForegroundBrightRed => Self::RED,
            SelectGraphicRendition::ForegroundBrightGreen => Self::GREEN,
            SelectGraphicRendition::ForegroundBrightYellow => Self::YELLOW,
            SelectGraphicRendition::ForegroundBrightBlue => Self::BLUE,
            SelectGraphicRendition::ForegroundBrightMagenta => Self::from_rgb(255, 0, 255),
            SelectGraphicRendition::ForegroundBrightCyan => Self::from_rgb(0, 255, 255),
            SelectGraphicRendition::ForegroundBrightWhite => Self::WHITE,
        }
    }
}
