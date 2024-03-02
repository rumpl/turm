use crate::grid::Grid;

#[derive(Debug, Default)]
struct CursorPos {
    x: usize,
    y: usize,
}

#[derive(Debug, Default)]
struct Cursor {
    pos: CursorPos,
}

#[derive(Debug)]
pub struct Turm {
    cursor: Cursor,
    grid: Grid,
    lines: usize,
    columns: usize,
}

impl Turm {
    pub fn new(lines: usize, columns: usize) -> Self {
        Self {
            cursor: Cursor::default(),
            grid: Grid::new(lines, columns),
            lines,
            columns,
        }
    }

    pub fn input(&mut self, c: u8) {
        self.grid[self.cursor.pos.x][self.cursor.pos.y].c = c as char;
        self.cursor.pos.x += 1;
        if self.cursor.pos.x == self.columns {
            self.cursor.pos.x = 0;
            self.cursor.pos.y += 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor.pos.x >= 1 {
            self.cursor.pos.x -= 1;
        }
    }
}
