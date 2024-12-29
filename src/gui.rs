use std::{
    io::Error,
    ops::DerefMut,
    os::fd::{AsRawFd, OwnedFd, RawFd},
    sync::{Arc, Mutex},
};

use crate::{font, terminal_gui_input::TerminalGuiInput, turm::Turm};
use egui::{Color32, FontFamily, FontId, Frame, Margin, Rect, Stroke};

fn resize(fd: RawFd, cols: usize, rows: usize, font_size: f32, width: f32) {
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
    font_size: f32,
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

        Self {
            terminal_gui_input,
            fd,
            turm,
            w: cols,
            h: rows,
            font_size: 12.0,
        }
    }
}

fn get_char_size(ctx: &egui::Context, font_size: f32) -> (f32, f32) {
    let font_id = FontId {
        size: font_size,
        family: FontFamily::Monospace, //Name("berkeley".into()),
    };
    ctx.fonts(move |fonts| {
        let rect = fonts
            .layout(
                "qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer\n\
                 qwerqwerqwerqwer"
                    .to_string(),
                font_id.clone(),
                Color32::WHITE,
                f32::INFINITY,
            )
            .rect;

        let w = rect.width() / 16.0;
        let h = rect.height() / 16.0;

        (w, h)
    })
}

impl eframe::App for TurmGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::CentralPanel::default();
        let frame = frame.frame(Frame {
            inner_margin: Margin {
                top: 0.0,
                left: 0.0,
                right: 0.0,
                ..Default::default()
            },
            ..Default::default()
        });
        frame.show(ctx, |ui| {
            let (width, height) = get_char_size(ctx, self.font_size);
            let w = (ui.available_width() / width) as usize;
            let h = (ui.available_height() / height) as usize;

            let mut turm1 = self.turm.lock().unwrap();
            let turm = turm1.deref_mut();

            if w != self.w || h != self.h {
                turm.grid.resize(w, h);
                turm.columns = w;
                turm.lines = h;
                self.w = w;
                self.h = h;

                resize(self.fd.as_raw_fd(), self.w, self.h, self.font_size, width);
            }

            ui.input(|input_state| {
                self.terminal_gui_input.write_input_to_terminal(input_state);
            });

            let font_id = FontId {
                size: self.font_size,
                family: FontFamily::Monospace, //Name("berkeley".into()),
            };
            let bold_font_id = FontId {
                size: self.font_size,
                family: FontFamily::Monospace, //Name("berkeley-bold".into()),
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
                    ..Default::default()
                };

                job.append(&section.text, 0.0, tf);
            }

            let res = ui.label(job);

            if turm.show_cursor {
                let painter = ui.painter();
                let pos = egui::pos2(
                    (turm.cursor.pos.x as f32) * width + res.rect.left(),
                    (turm.cursor.pos.y as f32) * height + res.rect.top() - 1.0,
                );
                let size = egui::vec2(width, height);
                painter.rect_filled(Rect::from_min_size(pos, size), 0.0, Color32::WHITE);
            }
            drop(turm1);
        });
    }
}
