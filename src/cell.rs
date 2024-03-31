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
            c: '.',
            fg: SelectGraphicRendition::ForegroundWhite,
            bg: SelectGraphicRendition::BackgroundBlack,
            wrap: false,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}
