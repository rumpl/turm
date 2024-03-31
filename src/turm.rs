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
    current_bg_color: SelectGraphicRendition,
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
            current_bg_color: SelectGraphicRendition::BackgroundBlack,
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
            self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = self.current_color;
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

    pub fn color(&mut self, c: SelectGraphicRendition) {
        println!("{:?}", c);
        match c {
            SelectGraphicRendition::ForegroundBlack => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundRed => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundGreen => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundYellow => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundBlue => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundMagenta => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundCyan => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundWhite => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundGrey => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundBrightRed => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundBrightGreen => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundBrightYellow => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundBrightBlue => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundBrightMagenta => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundBrightCyan => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundBrightWhite => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }
            SelectGraphicRendition::ForegroundRGB(_, _, _) => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].fg = c;
                self.current_color = c;
            }

            SelectGraphicRendition::BackgroundBlack => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundRed => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundGreen => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundYellow => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundBlue => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundMagenta => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundCyan => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundWhite => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundGrey => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundBrightRed => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundBrightGreen => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundBrightYellow => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundBrightBlue => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundBrightMagenta => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundBrightCyan => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
            SelectGraphicRendition::BackgroundBrightWhite => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].bg = c;
                self.current_bg_color = c;
            }
        };
    }

    pub fn clear_to_end_of_line(&mut self) {
        for i in self.cursor.pos.x..self.columns {
            self.grid[self.cursor.pos.y][i].c = ' ';
            self.grid[self.cursor.pos.y][i].fg = SelectGraphicRendition::ForegroundWhite;
            self.grid[self.cursor.pos.y][i].bg = SelectGraphicRendition::BackgroundBlack;
        }
    }

    pub fn clear_to_eos(&mut self) {
        let mut i = self.cursor.pos.x;
        let mut j = self.cursor.pos.y;
        loop {
            self.grid[j][i].c = ' ';
            self.grid[j][i].fg = SelectGraphicRendition::ForegroundWhite;
            self.grid[j][i].bg = SelectGraphicRendition::BackgroundBlack;
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
