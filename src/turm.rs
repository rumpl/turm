use crate::{ansi::SelectGraphicRendition, grid::Grid};

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
    current_color: SelectGraphicRendition,
    pub grid: Grid,
    lines: usize,
    columns: usize,
}

impl Turm {
    pub fn new(columns: usize, lines: usize) -> Self {
        Self {
            cursor: Cursor::default(),
            grid: Grid::new(columns, lines),
            current_color: SelectGraphicRendition::ForegroundWhite,
            lines,
            columns,
        }
    }

    pub fn input(&mut self, c: u8) {
        if c == b'\n' {
            self.cursor.pos.x = 0;
            self.cursor.pos.y += 1;
        } else if c == b'\r' {
            self.cursor.pos.x = 0;
        } else {
            self.grid[self.cursor.pos.y][self.cursor.pos.x].c = c as char;
            self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = self.current_color;
            self.cursor.pos.x += 1;
        }

        if self.cursor.pos.x == self.columns {
            self.cursor.pos.x = self.columns - 1;
        }
        if self.cursor.pos.y == self.lines {
            self.cursor.pos.y = self.lines - 1;
            self.cursor.pos.x = 0;
            self.grid.scroll_up();
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor.pos.x >= 1 {
            self.cursor.pos.x -= 1;
        }
    }

    pub fn color(&mut self, c: SelectGraphicRendition) {
        self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
        self.current_color = c;
    }
}
