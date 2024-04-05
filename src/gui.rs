use std::{
    os::fd::{AsRawFd, OwnedFd},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use crate::{
    ansi::{Ansi, AnsiOutput},
    color::Color,
    turm::Turm,
};
use egui::{
    Color32, Event, FontFamily, FontId, InputState, Key, Modifiers, Rect, Stroke, TextStyle,
};
use nix::ioctl_write_ptr_bad;

ioctl_write_ptr_bad!(
    set_window_size_ioctl,
    nix::libc::TIOCSWINSZ,
    nix::pty::Winsize
);

unsafe fn set_nonblocking(fd: i32) {
    use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};
    let _ = fcntl(fd, F_SETFL, fcntl(fd, F_GETFL, 0) | O_NONBLOCK);
}

pub struct TurmGui {
    turm: Turm,
    ansi: Ansi,
    rx: Receiver<Vec<u8>>,
    rtx: Sender<Vec<u8>>,
    show_cursor: bool,
}

impl TurmGui {
    pub fn new(cc: &eframe::CreationContext<'_>, fd: OwnedFd) -> Self {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "berkeley".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "/home/rumpl/.local/share/fonts/BerkeleyMono-Regular.ttf"
            )),
        );

        fonts.font_data.insert(
            "berkeley-bold".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "/home/rumpl/.local/share/fonts/BerkeleyMono-Bold.ttf"
            )),
        );

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "berkeley".to_owned());

        fonts.families.insert(
            FontFamily::Name("berkeley".into()),
            vec!["berkeley".to_owned()],
        );

        fonts.families.insert(
            FontFamily::Name("berkeley-bold".into()),
            vec!["berkeley-bold".to_owned()],
        );

        cc.egui_ctx.set_fonts(fonts);

        cc.egui_ctx.style_mut(|style| {
            style.override_text_style = Some(TextStyle::Monospace);
            for (_text_style, font_id) in style.text_styles.iter_mut() {
                font_id.size = 24.0;
            }
        });

        let cols: usize = 120;
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

        let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(0);
        let rs = cc.egui_ctx.clone();
        let write_fd = fd.try_clone().unwrap();
        // Thread that reads output from the shell and sends it to the gui
        thread::spawn(move || {
            unsafe {
                set_nonblocking(fd.as_raw_fd());
            }
            let poller = polling::Poller::new().unwrap();
            unsafe {
                poller
                    .add(fd.as_raw_fd(), polling::Event::readable(7))
                    .unwrap();
            }

            let mut events = polling::Events::new();
            let timeout = Duration::new(0, 10_000_000); // 10ms

            loop {
                let mut final_buf: Vec<u8> = vec![];
                let mut buf = vec![0u8; 1024];

                events.clear();

                poller.wait(&mut events, Some(timeout)).unwrap();
                // Immediately tell the poller that we are still interested in these events
                poller.modify(&fd, polling::Event::readable(7)).unwrap();

                let mut read = 0;
                // For some reason on MacOS this thing only returns data on the first read,
                // resulting in messages being sent in 1024 byte chunks, this makes redraw slow on
                // heavy UI applications like neovim, this works fine on Linux though.
                for _ in events.iter() {
                    for _ in 0..30 {
                        thread::sleep(Duration::new(0, 1_000));
                        let ret = nix::unistd::read(fd.as_raw_fd(), &mut buf);
                        //println!("{:?}", ret);
                        if let Ok(s) = ret {
                            //println!("read {s} bytes");
                            if s != 0 {
                                final_buf.append(&mut Vec::from(&mut buf[0..s]));
                                read += s;
                            } else {
                                //println!("done");
                                break;
                            }
                        }
                    }

                    rs.request_repaint();

                    //println!("read completely {read} bytes");
                    let _ = tx.send(final_buf[0..read].to_vec());
                }
            }
        });

        let (rtx, rrx) = mpsc::channel::<Vec<u8>>();
        // Thread that gets user input and sends it to the shell
        thread::spawn(move || loop {
            if let Ok(input) = rrx.recv() {
                let _ret = nix::unistd::write(write_fd.as_raw_fd(), &input);
            }
        });

        Self {
            rx,
            rtx,
            ansi: Ansi::new(),
            // TODO: calculate the right initial number of rows and columns
            turm: Turm::new(cols, rows),
            show_cursor: true,
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

    fn handle_input(&mut self) {
        let ret = self.rx.try_recv();
        if let Ok(buf) = ret {
            let ansi_res = self.ansi.push(&buf);
            for q in &ansi_res {
                match q {
                    AnsiOutput::Text(str) => {
                        for c in str {
                            self.turm.input(*c);
                        }
                    }
                    AnsiOutput::ClearToEndOfLine(_mode) => self.turm.clear_to_end_of_line(),
                    AnsiOutput::ClearToEOS => self.turm.clear_to_eos(),
                    AnsiOutput::MoveCursor(x, y) => self.turm.move_cursor(*x, *y),
                    AnsiOutput::MoveCursorHorizontal(x) => {
                        self.turm.move_cursor(*x, self.turm.cursor.pos.y)
                    }
                    AnsiOutput::CursorUp(amount) => self
                        .turm
                        .move_cursor(self.turm.cursor.pos.x, self.turm.cursor.pos.y - amount),
                    AnsiOutput::CursorDown(amount) => self
                        .turm
                        .move_cursor(self.turm.cursor.pos.x, self.turm.cursor.pos.y + amount),
                    AnsiOutput::CursorForward(amount) => self
                        .turm
                        .move_cursor(self.turm.cursor.pos.x + amount, self.turm.cursor.pos.y),
                    AnsiOutput::CursorBackward(amount) => {
                        if amount <= &self.turm.cursor.pos.x {
                            self.turm.move_cursor(
                                self.turm.cursor.pos.x - amount,
                                self.turm.cursor.pos.y,
                            );
                        }
                    }
                    AnsiOutput::HideCursor => self.show_cursor = false,
                    AnsiOutput::ShowCursor => self.show_cursor = true,
                    AnsiOutput::ScrollDown => self.turm.scroll_down(),
                    AnsiOutput::Backspace => self.turm.backspace(),
                    AnsiOutput::Sgr(c) => self.turm.color(*c),
                    AnsiOutput::Bell => println!("DING DONG"),
                }
            }
        };
    }
}

impl eframe::App for TurmGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font_size = 14.0;
        let line_height = font_size + 3.0;

        self.handle_input();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.input(|input_state| {
                self.write_input_to_terminal(input_state);
            });

            let font_id = FontId {
                size: font_size,
                family: FontFamily::Name("berkeley".into()),
            };
            let bold_font_id = FontId {
                size: font_size,
                family: FontFamily::Name("berkeley-bold".into()),
            };

            let mut job = egui::text::LayoutJob::default();
            for section in self.turm.grid.sections() {
                let fid = if section.style.bold {
                    bold_font_id.clone()
                } else {
                    font_id.clone()
                };

                let underline = Stroke {
                    color: section.style.fg.into(),
                    width: if section.style.underline { 4.0 } else { 0.0 },
                };

                let tf = egui::text::TextFormat {
                    font_id: fid,
                    color: section.style.fg.into(),
                    background: section.style.bg.into(),
                    underline,
                    italics: section.style.italics,
                    line_height: Some(line_height),
                    ..Default::default()
                };
                job.append(&section.text, 0.0, tf);
            }

            let res = ui.label(job);

            if self.show_cursor {
                let mut width = 0.0;
                ctx.fonts(|font| {
                    width = font.glyph_width(&font_id, 'm');
                });

                let painter = ui.painter();
                let pos = egui::pos2(
                    (self.turm.cursor.pos.x as f32) * width + res.rect.left(),
                    (self.turm.cursor.pos.y as f32) * line_height + res.rect.top(),
                );
                let size = egui::vec2(width, font_size);
                painter.rect_filled(Rect::from_min_size(pos, size), 0.0, Color32::WHITE);
            }
        });
    }
}

impl From<Color> for Color32 {
    fn from(c: Color) -> Self {
        Self::from_rgb(c.0[0], c.0[1], c.0[2])
    }
}
