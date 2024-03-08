use crate::grid::Grid;

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
    pub grid: Grid,
    lines: usize,
    columns: usize,
}

impl Turm {
    pub fn new(columns: usize, lines: usize) -> Self {
        Self {
            cursor: Cursor::default(),
            grid: Grid::new(columns, lines),
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
            // TODO: handle scroll here
            self.grid[self.cursor.pos.y][self.cursor.pos.x].c = c as char;
            self.cursor.pos.x += 1;
            if self.cursor.pos.x == self.columns {
                self.cursor.pos.x = 0;
                self.cursor.pos.y += 1;
            }
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor.pos.x >= 1 {
            self.cursor.pos.x -= 1;
        }
    }
}
