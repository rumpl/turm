use crate::{ansi::GraphicRendition, color::Color, grid::Grid};

#[derive(Debug, Default)]
pub struct CursorPos {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Default)]
pub struct Cursor {
    pub pos: CursorPos,
}

#[derive(Debug)]
pub struct Turm {
    pub cursor: Cursor,
    current_fg_color: Color,
    current_bg_color: Color,
    pub grid: Grid,
    lines: usize,
    columns: usize,
}

impl Turm {
    pub fn new(columns: usize, lines: usize) -> Self {
        Self {
            cursor: Cursor::default(),
            grid: Grid::new(columns, lines),
            current_fg_color: Color::WHITE,
            current_bg_color: Color::BLACK,
            lines,
            columns,
        }
    }

    pub fn input(&mut self, c: u8) {
        if c == b'\n' {
            self.move_cursor(0, self.cursor.pos.y + 1);
        } else if c == b'\r' {
            self.move_cursor(0, self.cursor.pos.y);
        } else {
            self.grid[self.cursor.pos.y][self.cursor.pos.x].c = c as char;
            self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = self.current_fg_color;
            self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = self.current_bg_color;
            self.move_cursor(self.cursor.pos.x + 1, self.cursor.pos.y);
        }

        if self.cursor.pos.x == self.columns {
            self.move_cursor(0, self.cursor.pos.y + 1);
        }

        if self.cursor.pos.y == self.lines {
            self.move_cursor(0, self.lines - 1);
            self.scroll_up();
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor.pos.x >= 1 {
            self.cursor.pos.x -= 1;
        }
    }

    pub fn color(&mut self, c: GraphicRendition) {
        match c {
            GraphicRendition::ForegroundColor(c) => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_fg_color = c;
            }
            GraphicRendition::BackgroundColor(c) => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
        };
    }

    pub fn clear_to_end_of_line(&mut self) {
        for i in self.cursor.pos.x..self.columns {
            self.grid[self.cursor.pos.y][i].c = ' ';
            self.grid[self.cursor.pos.y][i].fg = Color::WHITE;
            self.grid[self.cursor.pos.y][i].bg = Color::BLACK;
        }
    }

    pub fn clear_to_eos(&mut self) {
        let mut i = self.cursor.pos.x;
        let mut j = self.cursor.pos.y;
        loop {
            self.grid[j][i].c = ' ';
            self.grid[j][i].fg = Color::WHITE;
            self.grid[j][i].bg = Color::BLACK;
            i += 1;
            if i == self.columns {
                i = 0;
                j += 1;
            }
            if j == self.lines {
                break;
            }
        }
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) {
        if x > self.columns || y > self.lines {
            return;
        }
        self.cursor.pos.x = x;
        self.cursor.pos.y = y;
    }

    pub fn scroll_up(&mut self) {
        self.grid.scroll_up();
    }

    pub fn scroll_down(&mut self) {
        self.grid.scroll_down();
    }
}
