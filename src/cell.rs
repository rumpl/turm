use crate::ansi::SelectGraphicRendition;

#[derive(Debug, Copy, Clone)]
pub struct Cell {
    pub c: char,
    pub fg: SelectGraphicRendition,
    pub bg: SelectGraphicRendition,
    pub wrap: bool,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            c: ' ',
            fg: SelectGraphicRendition::ForegroundWhite,
            bg: SelectGraphicRendition::Reset, // TODO: add background colors
            wrap: false,
        }
    }
}
