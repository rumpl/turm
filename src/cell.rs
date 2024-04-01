use crate::color::Color;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub underline: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: Color::WHITE,
            bg: Color::BLACK,
            bold: false,
            underline: false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Cell {
    pub c: char,
    pub style: Style,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            c: ' ',
            style: Default::default(),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}
