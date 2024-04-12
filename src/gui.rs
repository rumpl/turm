use std::{
    ops::DerefMut,
    os::fd::RawFd,
    sync::{Arc, Mutex},
};

use crate::{font, terminal_gui_input::TerminalGuiInput, turm::Turm};
use egui::{Color32, FontFamily, FontId, Rect, Stroke};
use nix::ioctl_write_ptr_bad;

ioctl_write_ptr_bad!(
    set_window_size_ioctl,
    nix::libc::TIOCSWINSZ,
    nix::pty::Winsize
);

pub struct TurmGui {
    terminal_gui_input: TerminalGuiInput,
    turm: Arc<Mutex<Turm>>,
}

impl TurmGui {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        fd: RawFd,
        turm: Arc<Mutex<Turm>>,
        terminal_gui_input: TerminalGuiInput,
        cols: usize,
        rows: usize,
    ) -> Self {
        cc.egui_ctx.set_fonts(font::load());

        let ws = nix::pty::Winsize {
            ws_col: cols as u16,
            ws_row: rows as u16,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        unsafe {
            let _ = set_window_size_ioctl(fd, &ws);
        }

        Self {
            terminal_gui_input,
            turm,
        }
    }
}

impl eframe::App for TurmGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font_size = 10.0;
        let line_height = font_size + 3.0;

        egui::CentralPanel::default().show(ctx, |ui| {
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
