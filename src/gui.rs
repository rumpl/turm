use std::os::fd::{AsRawFd, OwnedFd};

use egui::{Color32, Event, FontFamily, FontId, InputState, Key, Rect, TextStyle};

use crate::{
    ansi::{Ansi, AnsiOutput, SelectGraphicRendition},
    turm::Turm,
};

#[derive(Debug, Default)]
pub struct CursorPos {
    x: usize,
    y: usize,
}

#[derive(Debug, Default)]
pub struct Cursor {
    pub pos: CursorPos,
}

pub struct TurmGui {
    buf: Vec<AnsiOutput>,
    // TODO: remove the cursor from here and use the one in turm
    cursor: Cursor,
    turm: Turm,
    fd: OwnedFd,
    ansi: Ansi,
}

impl TurmGui {
    pub fn new(cc: &eframe::CreationContext<'_>, fd: OwnedFd) -> Self {
        cc.egui_ctx.style_mut(|style| {
            style.override_text_style = Some(TextStyle::Monospace);
            for (_text_style, font_id) in style.text_styles.iter_mut() {
                font_id.size = 24.0;
            }
        });
        Self {
            fd,
            cursor: Cursor::default(),
            buf: vec![],
            ansi: Ansi::new(),
            // TODO: calculate the right initial number of rows and columns
            turm: Turm::new(40, 25),
        }
    }
}
impl eframe::App for TurmGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font_size = 24.0;
        let line_height = font_size + 4.0;

        let mut buf = vec![0u8; 4096];
        let ret = nix::unistd::read(self.fd.as_raw_fd(), &mut buf);

        if let Ok(read_size) = ret {
            let inc = &buf[0..read_size];
            let mut ansi_res = self.ansi.push(inc);
            for q in &ansi_res {
                if let AnsiOutput::Text(str) = q {
                    for c in str {
                        self.turm.input(*c);
                    }
                }
            }
            self.buf.append(&mut ansi_res);
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.input(|input_state| {
                write_input_to_terminal(input_state, &self.fd);
            });

            let font_id = FontId {
                size: font_size,
                family: FontFamily::Monospace,
            };

            let mut job = egui::text::LayoutJob::default();

            let mut color: Color32 = Color32::WHITE;
            job.append(
                &self.turm.grid.data(),
                0.0,
                egui::text::TextFormat {
                    font_id: font_id.clone(),
                    color,
                    line_height: Some(line_height),
                    ..Default::default()
                },
            );
            let _res = ui.label(job);
            return;

            self.cursor.pos.x = 0;
            self.cursor.pos.y = 0;
            for out in &self.buf {
                match out {
                    AnsiOutput::Text(str) => {
                        let text = String::from_utf8_lossy(&str);
                        for c in str {
                            match c {
                                b'\n' => {
                                    self.cursor.pos.x = 0;
                                    self.cursor.pos.y += 1;
                                }
                                _ => self.cursor.pos.x += 1,
                            }
                        }

                        job.append(
                            &text,
                            0.0,
                            egui::text::TextFormat {
                                font_id: font_id.clone(),
                                color,
                                line_height: Some(line_height),
                                ..Default::default()
                            },
                        );
                    }
                    AnsiOutput::SGR(c) => {
                        color = (*c).into();
                    }
                    AnsiOutput::Backspace => {
                        self.turm.backspace();
                        if self.cursor.pos.x >= 1 {
                            self.cursor.pos.x -= 1;
                        }
                    }
                    AnsiOutput::Bell => {
                        println!("ding");
                    }
                }
            }

            let res = ui.label(job);

            let mut width = 0.0;
            ctx.fonts(|font| {
                width = font.glyph_width(&font_id, 'i');
            });

            let painter = ui.painter();
            let pos = egui::pos2(
                (self.cursor.pos.x as f32) * width + res.rect.left(),
                (self.cursor.pos.y as f32) * line_height + res.rect.top(),
            );
            let size = egui::vec2(width, font_size);
            painter.rect_filled(Rect::from_min_size(pos, size), 0.0, Color32::WHITE);
        });
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
