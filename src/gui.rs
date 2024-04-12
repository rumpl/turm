use std::{
    ops::DerefMut,
    os::fd::{AsRawFd, OwnedFd},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{ansi::Ansi, color::Color, font, terminal_input::TerminalInput, turm::Turm};
use egui::{Color32, FontFamily, FontId, Rect, Stroke};
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
    terminal_input: TerminalInput,
    turm: Arc<Mutex<Turm>>,
}

impl TurmGui {
    pub fn new(cc: &eframe::CreationContext<'_>, fd: OwnedFd) -> Self {
        cc.egui_ctx.set_fonts(font::load());

        let cols: usize = 150;
        let rows: usize = 40;

        let ws = nix::pty::Winsize {
            ws_col: cols as u16,
            ws_row: rows as u16,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        unsafe {
            let _ = set_window_size_ioctl(fd.as_raw_fd(), &ws);
        }

        let rs = cc.egui_ctx.clone();
        let write_fd = fd.try_clone().unwrap();

        let turmie = Arc::new(Mutex::new(Turm::new(cols, rows)));

        // Thread that reads output from the shell and sends it to the gui
        let turm = turmie.clone();
        thread::spawn(move || {
            let mut ansi = Ansi::new();

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
                let mut buf = vec![0u8; 1024];

                events.clear();

                poller.wait(&mut events, Some(timeout)).unwrap();

                // For some reason on MacOS this thing only returns data on the first read,
                // resulting in messages being sent in 1024 byte chunks, this makes redraw slow on
                // heavy UI applications like neovim, this works fine on Linux though.
                let mut turm1 = turm.lock().unwrap();
                let turm = turm1.deref_mut();

                for _ in events.iter() {
                    for _ in 0..30 {
                        thread::sleep(Duration::new(0, 1_000));

                        let ret = nix::unistd::read(fd.as_raw_fd(), &mut buf);
                        if let Ok(s) = ret {
                            if s != 0 {
                                let ansi_res = ansi.push(&buf[0..s]);
                                turm.parse(ansi_res);
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    rs.request_repaint();
                }

                drop(turm1);

                poller.modify(&fd, polling::Event::readable(7)).unwrap();
            }
        });

        let (rtx, rrx) = mpsc::channel::<Vec<u8>>();
        // Thread that gets user input and sends it to the shell
        thread::spawn(move || loop {
            if let Ok(input) = rrx.recv() {
                _ = nix::unistd::write(write_fd.as_raw_fd(), &input);
            }
        });

        Self {
            terminal_input: TerminalInput::new(rtx),
            turm: turmie.clone(),
        }
    }
}

impl eframe::App for TurmGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font_size = 10.0;
        let line_height = font_size + 3.0;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.input(|input_state| {
                self.terminal_input.write_input_to_terminal(input_state);
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
            let mut turm1 = self.turm.lock().unwrap();
            let turm = turm1.deref_mut();
            for section in turm.grid.sections() {
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

            if turm.show_cursor {
                let mut width = 0.0;
                ctx.fonts(|font| {
                    width = font.glyph_width(&font_id, 'm');
                });

                let painter = ui.painter();
                let pos = egui::pos2(
                    (turm.cursor.pos.x as f32) * width + res.rect.left(),
                    (turm.cursor.pos.y as f32) * line_height
                        + (turm.cursor.pos.y as f32) * (-0.15)
                        + res.rect.top(),
                );
                let size = egui::vec2(width, font_size);
                painter.rect_filled(Rect::from_min_size(pos, size), 0.0, Color32::WHITE);
            }
            drop(turm1);
        });
    }
}

impl From<Color> for Color32 {
    fn from(c: Color) -> Self {
        Self::from_rgb(c.0[0], c.0[1], c.0[2])
    }
}
