use std::{
    ops::DerefMut,
    os::fd::{AsRawFd, OwnedFd},
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    ansi::Ansi, font, gui::Gui, terminal_gui_input::TerminalGuiInput, terminal_io::TerminalIO,
    turm::Turm,
};

use egui::{
    text::LayoutSection, Color32, FontFamily, FontId, Frame, Margin, Rect, Stroke, ViewportCommand,
};

pub struct EguiImpl {
    terminal_gui_input: TerminalGuiInput,
    turm: Arc<Mutex<Turm>>,
    w: usize,
    h: usize,
    fd: OwnedFd,
    font_size: f32,
}

impl Gui for EguiImpl {
    fn new(fd: OwnedFd, turm: Arc<Mutex<Turm>>, cols: usize, rows: usize) -> Self {
        let turm = turm.clone();
        let fd = fd.try_clone().unwrap();
        let write_fd = fd.try_clone().unwrap();

        let event_turm = turm.clone();
        let terminal_gui_input = TerminalGuiInput::new(event_turm, write_fd);

        Self {
            terminal_gui_input,
            fd,
            turm,
            w: cols,
            h: rows,
            font_size: 12.0,
        }
    }

    fn run(self) {
        // Create a context for egui to request repaints
        let egui_ctx = egui::Context::default();
        let rs = egui_ctx.clone();

        let turm = self.turm.clone();
        let fd = self.fd.try_clone().unwrap();

        // Thread that reads output from the shell and sends it to the gui
        thread::spawn(move || {
            let ansi = Ansi::new();
            let mut terminal_io = TerminalIO::new(ansi, fd, turm);
            terminal_io.start_io(|| {
                rs.request_repaint();
            });
        });

        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "Turm",
            native_options,
            Box::new(|cc| Ok(Box::new(self.with_creation_context(cc)))),
        )
        .expect("Failed to start eframe");
    }
}

impl EguiImpl {
    fn with_creation_context(self, cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_fonts(font::load());
        self
    }

    fn get_char_size(ctx: &egui::Context, font_size: f32) -> (f32, f32) {
        let font_id = FontId {
            size: font_size,
            family: FontFamily::Monospace,
        };
        ctx.fonts(move |fonts| {
            let mut job = egui::text::LayoutJob {
                round_output_size_to_nearest_ui_point: false,
                ..Default::default()
            };

            let tf = egui::text::TextFormat {
                font_id,
                line_height: Some(16.0),
                ..Default::default()
            };

            let text = "qwerqwerqwerqwer\n\
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
                    qwerqwerqwerqwer";
            job.append(text, 0.0, tf);

            let rect = fonts.layout_job(job).rect;
            let w = rect.width() / 16.0;
            let h = rect.height() / 16.0;

            (w, h)
        })
    }
}

impl eframe::App for EguiImpl {
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
            let (width, height) = Self::get_char_size(ctx, self.font_size);
            let w = (ui.available_width() / width) as usize;
            let h = (ui.available_height() / height) as usize;

            let mut turm1 = self.turm.lock().unwrap();
            let turm = turm1.deref_mut();

            if turm.title.is_empty() {
                ctx.send_viewport_cmd(ViewportCommand::Title(String::from("💩 Turm 💩")));
            } else {
                ctx.send_viewport_cmd(ViewportCommand::Title(
                    "💩 ".to_owned() + &turm.title.clone() + " 💩",
                ));
            }

            if w != self.w || h != self.h {
                turm.grid.resize(w, h);
                turm.columns = w;
                turm.lines = h;
                self.w = w;
                self.h = h;

                crate::gui::resize(self.fd.as_raw_fd(), self.w, self.h, self.font_size, width);
            }

            ui.input(|input_state| {
                self.terminal_gui_input.write_input_to_terminal(input_state);
            });

            let font_id = FontId {
                size: self.font_size,
                family: FontFamily::Monospace,
            };
            let bold_font_id = FontId {
                size: self.font_size,
                family: FontFamily::Monospace,
            };

            let sections = turm.grid.sections();
            let job = egui::text::LayoutJob {
                text: sections.text,
                sections: sections
                    .sections
                    .iter()
                    .map(|section| {
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
                            line_height: Some(16.0),
                            ..Default::default()
                        };

                        LayoutSection {
                            leading_space: 0.0,
                            byte_range: (section.offset..section.len),
                            format: tf,
                        }
                    })
                    .collect(),
                round_output_size_to_nearest_ui_point: false,
                ..Default::default()
            };

            let res = ui.label(job);

            if turm.show_cursor {
                let painter = ui.painter();
                let pos = egui::pos2(
                    (turm.cursor.pos.x as f32) * width + res.rect.left(),
                    (turm.cursor.pos.y as f32) * height + res.rect.top() - 1.0,
                );
                let size = egui::vec2(width, height);
                painter.rect_filled(Rect::from_min_size(pos, size), 0.0, Color32::WHITE);

                // Get character at cursor position and draw it in black on top of the cursor
                if turm.cursor.pos.y < turm.lines && turm.cursor.pos.x < turm.columns {
                    if let Some(c) = turm.grid[turm.cursor.pos.y][turm.cursor.pos.x].c {
                        painter.text(
                            pos,
                            egui::Align2::LEFT_TOP,
                            c.to_string(),
                            FontId {
                                size: self.font_size,
                                family: FontFamily::Monospace,
                            },
                            Color32::BLACK,
                        );
                    }
                }
            }
            drop(turm1);
        });
    }
}
