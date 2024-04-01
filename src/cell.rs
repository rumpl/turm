use crate::color::Color;

#[derive(Debug, Copy, Clone)]
pub struct Cell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub wrap: bool,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            c: '.',
            fg: Color::WHITE,
            bg: Color::BLACK,
            bold: false,
            wrap: false,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}
