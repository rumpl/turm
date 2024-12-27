use crate::{
    ansi::{AnsiOutput, GraphicRendition},
    grid::cell::Style,
    grid::Grid,
};

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
    pub show_cursor: bool,
    pub grid: Grid,

    current_style: Style,
    pub lines: usize,
    pub columns: usize,
    needs_wrap: bool,
}

impl Turm {
    pub fn new(columns: usize, lines: usize) -> Self {
        Self {
            cursor: Cursor::default(),
            grid: Grid::new(columns, lines),
            current_style: Style::default(),
            lines,
            columns,
            needs_wrap: false,
            show_cursor: true,
        }
    }

    pub fn parse(&mut self, ansi: Vec<AnsiOutput>) {
        for q in &ansi {
            match q {
                AnsiOutput::Text(str) => {
                    for c in str {
                        self.input(*c);
                    }
                }
                AnsiOutput::ClearToEndOfLine(_mode) => self.clear_to_end_of_line(),
                AnsiOutput::ClearToEOS => self.clear_to_eos(),
                AnsiOutput::MoveCursor(x, y) => self.move_cursor(*x, *y),
                AnsiOutput::MoveCursorHorizontal(x) => self.move_cursor(*x, self.cursor.pos.y),
                AnsiOutput::CursorUp(amount) => {
                    if self.cursor.pos.y >= *amount {
                        self.move_cursor(self.cursor.pos.x, self.cursor.pos.y - amount);
                    }
                }
                AnsiOutput::CursorDown(amount) => {
                    self.move_cursor(self.cursor.pos.x, self.cursor.pos.y + amount)
                }
                AnsiOutput::CursorForward(amount) => {
                    self.move_cursor(self.cursor.pos.x + amount, self.cursor.pos.y)
                }
                AnsiOutput::CursorBackward(amount) => {
                    if amount <= &self.cursor.pos.x {
                        self.move_cursor(self.cursor.pos.x - amount, self.cursor.pos.y);
                    }
                }
                AnsiOutput::HideCursor => self.show_cursor = false,
                AnsiOutput::ShowCursor => self.show_cursor = true,
                AnsiOutput::ScrollDown => self.scroll_down(),
                AnsiOutput::Backspace => self.backspace(),
                AnsiOutput::Sgr(c) => self.color(*c),
                AnsiOutput::Bell => println!("DING DONG"),
                AnsiOutput::FillWithE => self.fill_with_e(),
                AnsiOutput::NextLine => self.next_line(),
                AnsiOutput::DeleteCharacters(n) => self.delete_characters(*n),
            }
        }
    }

    fn delete_characters(&mut self, n: usize) {
        let x = self.cursor.pos.x;
        let mut y = self.cursor.pos.y;
        for _ in 0..n {
            self.grid[x][y].c = Some(' ');
            y += 1;
            if y == self.lines {
                self.next_line();
            }
        }
    }

    /// Fills the entier screen with 'E's, also known as DECALN
    /// https://www.vt100.net/docs/vt510-rm/DECALN.html
    fn fill_with_e(&mut self) {
        for i in 0..self.columns {
            for j in 0..self.lines {
                self.grid[j][i].c = Some('E');
                self.grid[j][i].style = Style::default();
            }
        }
        self.move_cursor(0, 0);
    }

    fn next_line(&mut self) {
        self.move_cursor(0, self.cursor.pos.y + 1);
        if self.cursor.pos.y == self.lines {
            self.move_cursor(0, self.lines - 1);
            self.scroll_up();
        }
    }

    pub fn input(&mut self, c: u8) {
        if c == b'\n' {
            self.move_cursor(self.cursor.pos.x, self.cursor.pos.y + 1);
            if self.cursor.pos.y == self.lines {
                self.needs_wrap = true;
            }
        } else if c == b'\r' {
            self.move_cursor(0, self.cursor.pos.y);
        } else if c == b'\t' {
            self.move_cursor(self.cursor.pos.x + 4, self.cursor.pos.y);
        } else {
            if self.needs_wrap {
                self.move_cursor(0, self.lines - 1);
                self.scroll_up();
                self.needs_wrap = false;
            }

            if self.cursor.pos.x + 1 > self.columns {
                self.move_cursor(0, self.cursor.pos.y + 1);
                if self.cursor.pos.y == self.lines {
                    self.move_cursor(0, self.lines - 1);
                    self.scroll_up();
                }
            }

            self.grid[self.cursor.pos.y][self.cursor.pos.x].c = Some(c as char);
            self.grid[self.cursor.pos.y][self.cursor.pos.x].style = self.current_style;

            self.move_cursor(self.cursor.pos.x + 1, self.cursor.pos.y);
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor.pos.x >= 1 {
            self.cursor.pos.x -= 1;
        }
    }

    pub fn color(&mut self, c: GraphicRendition) {
        if self.cursor.pos.x == self.columns || self.cursor.pos.y == self.lines {
            return;
        }

        match c {
            GraphicRendition::ForegroundColor(c) => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].style.fg = c;
                self.current_style.fg = c;
            }
            GraphicRendition::BackgroundColor(c) => {
                self.grid[self.cursor.pos.y][self.cursor.pos.x].style.bg = c;
                self.current_style.bg = c;
            }
            GraphicRendition::Bold => self.current_style.bold = true,
            GraphicRendition::Reset => {
                self.current_style = Style::default();
            }
            GraphicRendition::Underline => self.current_style.underline = true,
            GraphicRendition::Italic => self.current_style.italics = true,
        };
    }

    pub fn clear_to_end_of_line(&mut self) {
        for i in self.cursor.pos.x..self.columns {
            self.grid[self.cursor.pos.y][i].c = None;
            self.grid[self.cursor.pos.y][i].style = Style::default();
        }
    }

    pub fn clear_to_eos(&mut self) {
        let mut i = self.cursor.pos.x;
        let mut j = self.cursor.pos.y;
        loop {
            self.grid[j][i].c = None;
            self.grid[j][i].style = Style::default();
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
