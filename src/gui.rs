use std::{
    io::Error,
    ops::DerefMut,
    os::fd::{AsRawFd, OwnedFd, RawFd},
    sync::{Arc, Mutex},
};

use crate::{font, terminal_gui_input::TerminalGuiInput, turm::Turm};
use egui::{Color32, FontFamily, FontId, Rect, Stroke};

fn resize(fd: RawFd, cols: usize, rows: usize) {
    let ws = nix::pty::Winsize {
        ws_col: cols as u16,
        ws_row: rows as u16,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let res = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &ws as *const _) };
    if res < 0 {
        println!("ioctl TIOCSWINSZ failed: {}", Error::last_os_error());
        std::process::exit(1);
    }
}

fn resize2(fd: RawFd, cols: usize, rows: usize, font_size: f32, width: f32) {
    let ws = nix::pty::Winsize {
        ws_col: cols as u16,
        ws_row: rows as u16,
        ws_xpixel: cols as u16 * width as u16,
        ws_ypixel: rows as u16 * font_size as u16,
    };

    let res = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &ws as *const _) };
    if res < 0 {
        println!("ioctl TIOCSWINSZ failed: {}", Error::last_os_error());
        std::process::exit(1);
    }
}

pub struct TurmGui {
    terminal_gui_input: TerminalGuiInput,
    turm: Arc<Mutex<Turm>>,
    w: usize,
    h: usize,
    fd: OwnedFd,
}

impl TurmGui {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        fd: OwnedFd,
        turm: Arc<Mutex<Turm>>,
        terminal_gui_input: TerminalGuiInput,
        cols: usize,
        rows: usize,
    ) -> Self {
        cc.egui_ctx.set_fonts(font::load());

        resize(fd.as_raw_fd(), cols, rows);

        Self {
            terminal_gui_input,
            fd,
            turm,
            w: cols,
            h: rows,
        }
    }
}

impl eframe::App for TurmGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font_size = 14.0;
        let line_height = font_size + 3.0;
        let font_id = FontId {
            size: font_size,
            family: FontFamily::Name("berkeley".into()),
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut width = 0.0;
            ctx.fonts(|font| {
                width = font.glyph_width(&font_id, 'm');
            });

            let w = (ui.available_width() / width) as usize;
            let h = (ui.available_height() / line_height) as usize;

            let mut turm1 = self.turm.lock().unwrap();
            let turm = turm1.deref_mut();

            if w != self.w || h != self.h {
                turm.grid.resize(w, h);
                self.w = w;
                self.h = h;

                resize2(self.fd.as_raw_fd(), self.w, self.h, font_size, width);
            }

            ui.input(|input_state| {
                self.terminal_gui_input.write_input_to_terminal(input_state);
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
