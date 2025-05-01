use std::{
    ops::DerefMut,
    os::fd::{AsRawFd, OwnedFd},
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    ansi::Ansi, gui::Gui, terminal_gui_input::TerminalGuiInput, terminal_io::TerminalIO, turm::Turm,
};

use egui;
use glib;
use gtk::prelude::*;
use gtk4 as gtk;
use pango;
use pangocairo;

// Terminal selection
#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Clone, Debug)]
struct Selection {
    start: Position,
    end: Position,
    active: bool,
}

impl Selection {
    fn new() -> Self {
        Self {
            start: Position { x: 0, y: 0 },
            end: Position { x: 0, y: 0 },
            active: false,
        }
    }

    fn clear(&mut self) {
        self.active = false;
    }

    fn is_position_selected(&self, x: usize, y: usize) -> bool {
        if !self.active {
            return false;
        }

        let (start, end) = self.normalized();

        if y < start.y || y > end.y {
            return false;
        }

        if y == start.y && y == end.y {
            return x >= start.x && x < end.x;
        } else if y == start.y {
            return x >= start.x;
        } else if y == end.y {
            return x < end.x;
        }

        true
    }

    fn normalized(&self) -> (Position, Position) {
        if self.start.y > self.end.y || (self.start.y == self.end.y && self.start.x > self.end.x) {
            (self.end, self.start)
        } else {
            (self.start, self.end)
        }
    }

    fn get_selected_text(&self, terminal: &Turm) -> String {
        if !self.active {
            return String::new();
        }

        let (start, end) = self.normalized();
        let mut result = String::new();

        for y in start.y..=end.y {
            if y >= terminal.lines {
                break;
            }

            let start_x = if y == start.y { start.x } else { 0 };
            let end_x = if y == end.y {
                std::cmp::min(end.x, terminal.columns)
            } else {
                terminal.columns
            };

            for x in start_x..end_x {
                if x >= terminal.columns {
                    break;
                }

                if let Some(c) = terminal.grid[y][x].c {
                    result.push(c);
                }
            }

            // Add newline if not the last line
            if y < end.y {
                result.push('\n');
            }
        }

        result
    }
}

pub struct Gtk4Impl {
    terminal_gui_input: TerminalGuiInput,
    turm: Arc<Mutex<Turm>>,
    w: usize,
    h: usize,
    fd: OwnedFd,
    font_size: f32,
    selection: Arc<Mutex<Selection>>,
}

impl Gui for Gtk4Impl {
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
            selection: Arc::new(Mutex::new(Selection::new())),
        }
    }

    fn run(self) {
        // Initialize GTK
        gtk::init().expect("Failed to initialize GTK");

        let turm_clone = self.turm.clone();
        let fd_clone = self.fd.try_clone().unwrap();
        let font_size = self.font_size;
        let terminal_gui_input = self.terminal_gui_input.clone();
        let selection = self.selection.clone();

        // Create a channel for terminal updates
        let (tx, rx) = glib::MainContext::channel::<()>(glib::Priority::DEFAULT);
        // Create a drawing area for terminal content
        let drawing_area = gtk::DrawingArea::new();

        // Setup drawing area redraw
        let da = drawing_area.clone();
        rx.attach(None, move |_| {
            da.queue_draw();
            glib::ControlFlow::Continue
        });

        // Thread that reads output from the shell and sends it to the gui
        let turm_io = turm_clone.clone();
        let fd_io = fd_clone.try_clone().unwrap();
        thread::spawn(move || {
            let ansi = Ansi::new();
            let mut terminal_io = TerminalIO::new(ansi, fd_io, turm_io);
            terminal_io.start_io(|| {
                // Signal that the terminal has updated
                let _ = tx.send(());
            });
        });

        // Create the application
        let app = gtk::Application::new(Some("org.turm.terminal"), Default::default());

        // Setup the application activate signal
        app.connect_activate(move |app| {
            // Create a window
            let window = gtk::ApplicationWindow::new(app);
            window.set_title(Some("Turm"));
            window.set_default_size(800, 600);

            window.set_child(Some(&drawing_area));

            // Clone needed values for the draw callback
            let turm = turm_clone.clone();
            let fd_clone = fd_clone.try_clone().unwrap();
            let selection_for_draw = selection.clone();

            let window_clone = window.clone();
            // Setup drawing callback
            drawing_area.set_draw_func(move |_, cr, width, height| {
                // Fill the background with black
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.rectangle(0.0, 0.0, width as f64, height as f64);
                let _ = cr.fill();

                let mut turm_lock = turm.lock().unwrap();
                let terminal = turm_lock.deref_mut();
                let selection_lock = selection_for_draw.lock().unwrap();

                // Calculate character dimensions
                let mut font_desc = pango::FontDescription::new();
                font_desc.set_family("Monospace");
                font_desc.set_size((font_size * pango::SCALE as f32) as i32);

                let layout = pangocairo::create_layout(cr);
                layout.set_font_description(Some(&font_desc));
                layout.set_text("X");

                let (_, logical_rect) = layout.pixel_extents();
                let char_width = logical_rect.width() as f32;
                let char_height = logical_rect.height() as f32;

                // Calculate terminal size based on window size
                let w = (width as f32 / char_width) as usize;
                let h = (height as f32 / char_height) as usize;

                // Resize the terminal if needed
                if w != terminal.columns || h != terminal.lines {
                    terminal.grid.resize(w, h);
                    terminal.columns = w;
                    terminal.lines = h;

                    crate::gui::resize(fd_clone.as_raw_fd(), w, h, font_size, char_width);
                }

                // Set the window title
                if terminal.title.is_empty() {
                    window_clone.set_title(Some("💩 Turm 💩"));
                } else {
                    window_clone.set_title(Some(&format!("💩 {} 💩", terminal.title)));
                }

                // Render terminal content
                let sections = terminal.grid.sections();
                // println!("{}", terminal.grid);
                let mut current_row = 0;
                let mut current_col = 0;
                for section in &sections.sections {
                    // Set colors
                    let fg = section.style.fg;
                    let bg = section.style.bg;

                    // Set foreground color based on the section style
                    cr.set_source_rgb(
                        fg.0[0] as f64 / 255.0,
                        fg.0[1] as f64 / 255.0,
                        fg.0[2] as f64 / 255.0,
                    );

                    // Get text for this section
                    let section_text = &sections.text[section.offset..section.len];

                    // Calculate starting position
                    let x = (current_col as f32 * char_width) as f64;
                    let y = (current_row as f32 * char_height) as f64;

                    // Draw background if needed
                    if bg.0[0] != 0 || bg.0[1] != 0 || bg.0[2] != 0 {
                        // Set background color
                        cr.set_source_rgb(
                            bg.0[0] as f64 / 255.0,
                            bg.0[1] as f64 / 255.0,
                            bg.0[2] as f64 / 255.0,
                        );

                        // Draw backgrounds line by line
                        let mut current_x = x;
                        let mut current_y = y;
                        let mut chars_in_line = 0;

                        for c in section_text.chars() {
                            if c == '\n' || chars_in_line == terminal.columns {
                                // Draw background for this line
                                let width = chars_in_line as f64 * char_width as f64;
                                cr.rectangle(current_x, current_y, width, char_height as f64);
                                cr.fill().expect("Failed to fill background");

                                // Move to next line
                                current_y += char_height as f64;
                                current_x = x;
                                chars_in_line = 0;
                            } else {
                                chars_in_line += 1;
                            }
                        }

                        // Draw background for the last line
                        if chars_in_line > 0 {
                            let width = chars_in_line as f64 * char_width as f64;
                            cr.rectangle(current_x, current_y, width, char_height as f64);
                            cr.fill().expect("Failed to fill background");
                        }

                        // Reapply foreground color
                        cr.set_source_rgb(
                            fg.0[0] as f64 / 255.0,
                            fg.0[1] as f64 / 255.0,
                            fg.0[2] as f64 / 255.0,
                        );
                    }

                    // Create attributes for text styling
                    let attr_list = pango::AttrList::new();
                    if section.style.bold {
                        let attr = pango::AttrInt::new_weight(pango::Weight::Bold);
                        attr_list.insert(attr);
                    }
                    if section.style.italics {
                        let attr = pango::AttrInt::new_style(pango::Style::Italic);
                        attr_list.insert(attr);
                    }
                    if section.style.underline {
                        let attr = pango::AttrInt::new_underline(pango::Underline::Single);
                        attr_list.insert(attr);
                    }

                    // Process text character by character for precise positioning
                    let mut current_x = x;
                    let mut current_y = y;
                    let mut col = current_col;
                    let mut row = current_row;
                    let mut line_count = 0;
                    let mut last_line_chars = 0;

                    // Split text into lines first
                    let lines = section_text.split('\n');

                    for (line_idx, line) in lines.enumerate() {
                        if line_idx > 0 {
                            // For subsequent lines after a newline, reset to column 0
                            current_x = 0.0;
                            current_y += char_height as f64;
                            col = 0;
                            row += 1;
                            line_count += 1;
                        }

                        // If the line is empty (just a newline), skip further processing
                        if line.is_empty() {
                            last_line_chars = 0;
                            continue;
                        }

                        // Draw the current line
                        cr.move_to(current_x, current_y);
                        let line_layout = pangocairo::create_layout(cr);
                        line_layout.set_font_description(Some(&font_desc));
                        line_layout.set_attributes(Some(&attr_list));

                        // Check if this line needs wrapping
                        if col + line.chars().count() > terminal.columns {
                            // Handle wrapping - draw only up to the end of the current terminal row
                            let mut chars_done = 0;

                            // Process the line in chunks that fit within terminal width
                            let line_chars = line.chars().collect::<Vec<_>>();

                            while chars_done < line_chars.len() {
                                let remaining_cols = if chars_done == 0 {
                                    terminal.columns - col
                                } else {
                                    terminal.columns
                                };

                                let chunk_size =
                                    std::cmp::min(remaining_cols, line_chars.len() - chars_done);
                                let chunk: String = line_chars[chars_done..chars_done + chunk_size]
                                    .iter()
                                    .collect();

                                // Draw this chunk
                                let chunk_x = if chars_done == 0 { current_x } else { 0.0 };
                                let current_chunk_row = row;
                                let current_chunk_col = if chars_done == 0 { col } else { 0 };

                                // Check for selection and highlight if needed
                                for (i, _) in chunk.chars().enumerate() {
                                    let char_x = chunk_x
                                        + (i + current_chunk_col) as f64 * char_width as f64;
                                    if selection_lock.is_position_selected(
                                        current_chunk_col + i,
                                        current_chunk_row,
                                    ) {
                                        // Draw selection highlight
                                        cr.set_source_rgb(0.5, 0.5, 1.0); // Selection color (light blue)
                                        cr.rectangle(
                                            char_x,
                                            current_y,
                                            char_width as f64,
                                            char_height as f64,
                                        );
                                        cr.fill().expect("Failed to fill selection highlight");
                                    }
                                }

                                // Restore foreground color
                                cr.set_source_rgb(
                                    fg.0[0] as f64 / 255.0,
                                    fg.0[1] as f64 / 255.0,
                                    fg.0[2] as f64 / 255.0,
                                );

                                cr.move_to(chunk_x, current_y);

                                let chunk_layout = pangocairo::create_layout(cr);
                                chunk_layout.set_font_description(Some(&font_desc));
                                chunk_layout.set_attributes(Some(&attr_list));
                                chunk_layout.set_text(&chunk);
                                pangocairo::show_layout(cr, &chunk_layout);

                                // Move to next line if needed
                                chars_done += chunk_size;
                                if chars_done < line_chars.len() {
                                    current_y += char_height as f64;
                                    row += 1;
                                    line_count += 1;
                                }
                            }

                            // Update last_line_chars to the remaining characters on the last line
                            last_line_chars = (line.chars().count() - (terminal.columns - col))
                                % terminal.columns;
                            if last_line_chars == 0 && line.chars().count() > 0 {
                                last_line_chars = terminal.columns;
                            }
                        } else {
                            // Check for selection and highlight if needed
                            for (i, _) in line.chars().enumerate() {
                                let char_x = current_x + (i + col) as f64 * char_width as f64;
                                if selection_lock.is_position_selected(col + i, row) {
                                    // Draw selection highlight
                                    cr.set_source_rgb(0.5, 0.5, 1.0); // Selection color (light blue)
                                    cr.rectangle(
                                        char_x,
                                        current_y,
                                        char_width as f64,
                                        char_height as f64,
                                    );
                                    cr.fill().expect("Failed to fill selection highlight");
                                }
                            }

                            // Restore foreground color after any highlights
                            cr.set_source_rgb(
                                fg.0[0] as f64 / 255.0,
                                fg.0[1] as f64 / 255.0,
                                fg.0[2] as f64 / 255.0,
                            );

                            cr.move_to(current_x, current_y);
                            line_layout.set_text(line);
                            pangocairo::show_layout(cr, &line_layout);
                            last_line_chars = col + line.chars().count();
                        }
                    }

                    // Update current_row and current_col for next section
                    current_row += line_count;
                    current_col = last_line_chars % terminal.columns;
                }

                // Draw cursor if visible
                if terminal.show_cursor {
                    let cursor_x = terminal.cursor.pos.x as f32 * char_width;
                    let cursor_y = terminal.cursor.pos.y as f32 * char_height;

                    // Draw cursor rectangle
                    cr.set_source_rgb(1.0, 1.0, 1.0); // White
                    cr.rectangle(
                        cursor_x as f64,
                        cursor_y as f64,
                        char_width as f64,
                        char_height as f64,
                    );
                    let _ = cr.fill();

                    // Draw character at cursor position in black
                    if terminal.cursor.pos.y < terminal.lines
                        && terminal.cursor.pos.x < terminal.columns
                    {
                        if let Some(c) =
                            terminal.grid[terminal.cursor.pos.y][terminal.cursor.pos.x].c
                        {
                            cr.set_source_rgb(0.0, 0.0, 0.0); // Black

                            let cursor_layout = pangocairo::create_layout(cr);
                            cursor_layout.set_font_description(Some(&font_desc));
                            cursor_layout.set_text(&c.to_string());

                            cr.move_to(cursor_x as f64, cursor_y as f64);
                            pangocairo::show_layout(cr, &cursor_layout);
                        }
                    }
                }
            });

            // Setup keyboard event controller
            let key_controller = gtk::EventControllerKey::new();

            // Clone the terminal_gui_input for the key handlers
            let terminal_gui_input_pressed = terminal_gui_input.clone();
            key_controller.connect_key_pressed(move |_controller, key, _keycode, state| {
                // Convert GTK key event to egui InputState
                let modifiers = egui::Modifiers {
                    alt: state.contains(gtk::gdk::ModifierType::ALT_MASK),
                    ctrl: state.contains(gtk::gdk::ModifierType::CONTROL_MASK),
                    shift: state.contains(gtk::gdk::ModifierType::SHIFT_MASK),
                    mac_cmd: false, // GTK on non-Mac platforms doesn't use this
                    command: state.contains(gtk::gdk::ModifierType::CONTROL_MASK), // On non-Mac, command is ctrl
                };

                // Convert the GTK key to an egui Key
                let egui_key = match key {
                    gtk::gdk::Key::Return => egui::Key::Enter,
                    gtk::gdk::Key::BackSpace => egui::Key::Backspace,
                    gtk::gdk::Key::Tab => egui::Key::Tab,
                    gtk::gdk::Key::Escape => egui::Key::Escape,
                    gtk::gdk::Key::Up => egui::Key::ArrowUp,
                    gtk::gdk::Key::Down => egui::Key::ArrowDown,
                    gtk::gdk::Key::Left => egui::Key::ArrowLeft,
                    gtk::gdk::Key::Right => egui::Key::ArrowRight,
                    // Add more key mappings as needed
                    _ => {
                        // Convert regular keys by their name
                        if let Some(c) = key.to_unicode() {
                            let mut input = egui::InputState::default();
                            let event = egui::Event::Text(c.to_string());
                            let events = vec![event];

                            input.events = events;
                            input.modifiers = modifiers;

                            terminal_gui_input_pressed.write_input_to_terminal(&input);
                            return glib::Propagation::Stop;
                        } else {
                            return glib::Propagation::Proceed;
                        }
                    }
                };

                // Create key press event
                let key_event = egui::Event::Key {
                    key: egui_key,
                    physical_key: None, // We don't have this information
                    pressed: true,
                    repeat: false, // This information is not in the parameters
                    modifiers,
                };

                // Create input state with our event
                let mut input_state = egui::InputState::default();
                let events = vec![key_event];
                input_state.events = events;
                input_state.modifiers = modifiers;

                // Pass the input to terminal_gui_input
                terminal_gui_input_pressed.write_input_to_terminal(&input_state);

                glib::Propagation::Stop
            });

            window.add_controller(key_controller);

            // Add mouse controller for selection
            let mouse_controller = gtk::EventControllerMotion::new();
            let selection_clone = selection.clone();
            let drawing_area_clone = drawing_area.clone();

            // Track mouse button state
            let gesture_click = gtk::GestureClick::new();
            gesture_click.set_button(1); // Left button

            // Clone variables for the first closure
            let selection_clone1 = selection_clone.clone();
            let drawing_area_clone1 = drawing_area_clone.clone();

            gesture_click.connect_pressed(move |_, _n_press, x, y| {
                let mut selection_lock = selection_clone1.lock().unwrap();

                // Calculate character dimensions (should match drawing code)
                let char_width = width_for_font_size(font_size);
                let char_height = height_for_font_size(font_size);

                // Convert mouse coordinates to terminal coordinates
                let term_x = (x / char_width as f64) as usize;
                let term_y = (y / char_height as f64) as usize;

                // Update selection start
                selection_lock.start = Position {
                    x: term_x,
                    y: term_y,
                };
                selection_lock.end = Position {
                    x: term_x,
                    y: term_y,
                };
                selection_lock.active = true;

                // Redraw to show selection
                drawing_area_clone1.queue_draw();
            });

            // Clone variables for the motion closure
            let selection_clone2 = selection_clone.clone();
            let drawing_area_clone2 = drawing_area_clone.clone();

            mouse_controller.connect_motion(move |_, x, y| {
                let mut selection_lock = selection_clone2.lock().unwrap();

                // Only update if selection is active (mouse button is pressed)
                if selection_lock.active {
                    // Calculate character dimensions (should match drawing code)
                    let char_width = width_for_font_size(font_size);
                    let char_height = height_for_font_size(font_size);

                    // Convert mouse coordinates to terminal coordinates
                    let term_x = (x / char_width as f64) as usize;
                    let term_y = (y / char_height as f64) as usize;

                    // Update selection end
                    selection_lock.end = Position {
                        x: term_x,
                        y: term_y,
                    };

                    // Redraw to show updated selection
                    drawing_area_clone2.queue_draw();
                }
            });

            let gesture_click_release = gtk::GestureClick::new();
            gesture_click_release.set_button(1); // Left button

            // Get clipboard for copy operations
            let clipboard = window.clipboard();
            // Clone clipboard early for use in multiple closures
            let clipboard_for_release = clipboard.clone();
            let clipboard_for_context = clipboard.clone();

            let turm_copy = turm_clone.clone();
            let selection_copy = selection.clone();
            let drawing_area_clone3 = drawing_area_clone.clone();

            gesture_click_release.connect_released(move |_, _n_press, _x, _y| {
                let selection_lock = selection_copy.lock().unwrap();
                let turm_lock = turm_copy.lock().unwrap();

                // If this was a click without drag, clear selection
                if selection_lock.start.x == selection_lock.end.x
                    && selection_lock.start.y == selection_lock.end.y
                {
                    // This was just a click without drag, clear selection
                    std::mem::drop(selection_lock);
                    selection_copy.lock().unwrap().clear();
                } else {
                    // Get selected text and copy to clipboard if Ctrl is pressed
                    let selected_text = selection_lock.get_selected_text(&turm_lock);

                    // Automatically copy selection to clipboard
                    if !selected_text.is_empty() {
                        clipboard_for_release.set_text(&selected_text);
                    }
                }

                drawing_area_clone3.queue_draw();
            });

            // Add context menu for copy
            let gesture_right_click = gtk::GestureClick::new();
            gesture_right_click.set_button(3); // Right button

            let turm_context = turm_clone.clone();
            let selection_context = selection.clone();
            let drawing_area_clone4 = drawing_area_clone.clone();

            gesture_right_click.connect_pressed(move |_, _n_press, x, y| {
                let selection_lock = selection_context.lock().unwrap();
                let turm_lock = turm_context.lock().unwrap();

                if selection_lock.active {
                    let selected_text = selection_lock.get_selected_text(&turm_lock);
                    if !selected_text.is_empty() {
                        let menu = gtk::PopoverMenu::builder().build();
                        let box_layout = gtk::Box::new(gtk::Orientation::Vertical, 5);

                        let copy_button = gtk::Button::with_label("Copy");
                        let clipboard_copy = clipboard_for_context.clone();
                        let text_to_copy = selected_text.clone();

                        copy_button.connect_clicked(move |_| {
                            clipboard_copy.set_text(&text_to_copy);
                        });

                        box_layout.append(&copy_button);
                        menu.set_child(Some(&box_layout));

                        // Set the parent using the correct method for GTK4
                        menu.set_parent(&drawing_area_clone4);
                        menu.set_pointing_to(Some(&gtk::gdk::Rectangle::new(
                            x as i32, y as i32, 1, 1,
                        )));
                        menu.popup();
                    }
                }
            });

            // Add the event controllers to the drawing area
            drawing_area.add_controller(mouse_controller);
            drawing_area.add_controller(gesture_click);
            drawing_area.add_controller(gesture_click_release);
            drawing_area.add_controller(gesture_right_click);

            // Show all widgets
            window.show();
        });

        // Run the GTK application
        app.run();
    }
}

// Helper function to calculate width for font size
fn width_for_font_size(font_size: f32) -> f32 {
    // This is an approximation - for a monospace font, width is typically around 60% of the font size
    font_size * 0.6
}

// Helper function to calculate height for font size
fn height_for_font_size(font_size: f32) -> f32 {
    // For most monospace fonts, the height is about 1.2-1.3 times the font size
    font_size // * 1.2
}
