use crate::ansi_codes;

fn is_csi_terminator(b: u8) -> bool {
    (0x40..=0x7e).contains(&b)
}

fn is_csi_param(b: u8) -> bool {
    (0x30..=0x3f).contains(&b)
}

fn is_csi_intermediate(b: u8) -> bool {
    (0x20..=0x2f).contains(&b)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CSIParserState {
    Parameters,
    Intermediates,
}

#[derive(Debug)]
enum CSIParserError {
    InvalidCSI,
}

#[derive(Debug, Default, Clone)]
struct CSIParseResult {
    params: Vec<u8>,
    intermediates: Vec<u8>,
    func: u8,
}

#[derive(Debug)]
struct CSIParser {
    result: CSIParseResult,
    state: CSIParserState,
}

impl CSIParser {
    fn new() -> Self {
        Self {
            state: CSIParserState::Parameters,
            result: CSIParseResult::default(),
        }
    }

    fn push(&mut self, b: u8) -> Option<Result<CSIParseResult, CSIParserError>> {
        if is_csi_param(b) {
            if self.state == CSIParserState::Intermediates {
                return Some(Err(CSIParserError::InvalidCSI));
            }
            self.result.params.push(b);
        } else if is_csi_intermediate(b) {
            self.result.intermediates.push(b);
            self.state = CSIParserState::Intermediates;
        } else if is_csi_terminator(b) {
            self.result.func = b;
            return Some(Ok(self.result.clone()));
        }

        None
    }
}

enum AnsiState {
    Empty,
    Escape,
    Csi(CSIParser),
}

pub struct Ansi {
    state: AnsiState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub [u8; 3]);

impl Default for Color {
    fn default() -> Self {
        Self([0, 0, 0])
    }
}

impl Color {
    pub const BLACK: Self = Self::from_rgb(0, 0, 0);
    pub const DARK_GRAY: Self = Self::from_rgb(96, 96, 96);
    pub const GRAY: Self = Self::from_rgb(160, 160, 160);
    pub const LIGHT_GRAY: Self = Self::from_rgb(220, 220, 220);
    pub const WHITE: Self = Self::from_rgb(255, 255, 255);
    pub const MAGENTA: Self = Self::from_rgb(170, 0, 170);
    pub const CYAN: Self = Self::from_rgb(0, 170, 170);

    pub const BROWN: Self = Self::from_rgb(165, 42, 42);
    pub const DARK_RED: Self = Self::from_rgb(0x8B, 0, 0);
    pub const RED: Self = Self::from_rgb(255, 0, 0);
    pub const LIGHT_RED: Self = Self::from_rgb(255, 128, 128);

    pub const YELLOW: Self = Self::from_rgb(255, 255, 0);
    pub const LIGHT_YELLOW: Self = Self::from_rgb(255, 255, 0xE0);
    pub const KHAKI: Self = Self::from_rgb(240, 230, 140);

    pub const DARK_GREEN: Self = Self::from_rgb(0, 0x64, 0);
    pub const GREEN: Self = Self::from_rgb(0, 255, 0);
    pub const LIGHT_GREEN: Self = Self::from_rgb(0x90, 0xEE, 0x90);

    pub const DARK_BLUE: Self = Self::from_rgb(0, 0, 0x8B);
    pub const BLUE: Self = Self::from_rgb(0, 0, 255);
    pub const LIGHT_BLUE: Self = Self::from_rgb(0xAD, 0xD8, 0xE6);

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b])
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GraphicRendition {
    BackgroundColor(Color),
    ForegroundColor(Color),
    //Bold,
}

impl From<u8> for GraphicRendition {
    fn from(item: u8) -> Self {
        match item {
            30 => Self::ForegroundColor(Color::BLACK),
            31 => Self::ForegroundColor(Color::RED),
            32 => Self::ForegroundColor(Color::GREEN),
            33 => Self::ForegroundColor(Color::YELLOW),
            34 => Self::ForegroundColor(Color::BLUE),
            35 => Self::ForegroundColor(Color::MAGENTA),
            36 => Self::ForegroundColor(Color::CYAN),
            37 => Self::ForegroundColor(Color::WHITE),

            /*
                        90 => Self::ForegroundGrey,
                        91 => Self::ForegroundBrightRed,
                        92 => Self::ForegroundBrightGreen,
                        93 => Self::ForegroundBrightYellow,
                        94 => Self::ForegroundBrightBlue,
                        95 => Self::ForegroundBrightMagenta,
                        96 => Self::ForegroundBrightCyan,
                        97 => Self::ForegroundBrightWhite,
            */
            40 => Self::BackgroundColor(Color::BLACK),
            41 => Self::BackgroundColor(Color::RED),
            42 => Self::BackgroundColor(Color::GREEN),
            43 => Self::BackgroundColor(Color::YELLOW),
            44 => Self::BackgroundColor(Color::BLUE),
            45 => Self::BackgroundColor(Color::MAGENTA),
            46 => Self::BackgroundColor(Color::CYAN),
            47 => Self::BackgroundColor(Color::WHITE),

            /*
                    100 => Self::BackgroundGrey,
                    101 => Self::BackgroundBrightRed,
                    102 => Self::BackgroundBrightGreen,
                    103 => Self::BackgroundBrightYellow,
                    104 => Self::BackgroundBrightBlue,
                    105 => Self::BackgroundBrightMagenta,
                    106 => Self::BackgroundBrightCyan,
                    107 => Self::BackgroundBrightWhite,
            */
            _ => panic!("unknown sgr {}", item),
        }
    }
}

fn color_8bit(item: u8) -> GraphicRendition {
    match item {
        0 => GraphicRendition::ForegroundColor(Color::BLACK),
        1 => GraphicRendition::ForegroundColor(Color::RED),
        2 => GraphicRendition::ForegroundColor(Color::GREEN),
        3 => GraphicRendition::ForegroundColor(Color::YELLOW),
        4 => GraphicRendition::ForegroundColor(Color::BLUE),
        5 => GraphicRendition::ForegroundColor(Color::MAGENTA),
        6 => GraphicRendition::ForegroundColor(Color::CYAN),
        7 => GraphicRendition::ForegroundColor(Color::WHITE),

        8 => GraphicRendition::ForegroundColor(Color::GRAY),
        9 => GraphicRendition::ForegroundColor(Color::RED),
        10 => GraphicRendition::ForegroundColor(Color::GREEN),
        11 => GraphicRendition::ForegroundColor(Color::YELLOW),
        12 => GraphicRendition::ForegroundColor(Color::BLUE),
        13 => GraphicRendition::ForegroundColor(Color::MAGENTA),
        14 => GraphicRendition::ForegroundColor(Color::CYAN),
        15 => GraphicRendition::ForegroundColor(Color::WHITE),

        16..=231 => GraphicRendition::ForegroundColor(Color::from_rgb(
            (item - 16) & 0b1110_0000,
            (item - 16) & 0b0001_1100,
            (item - 16) & 0b0000_0011,
        )),
        _ => panic!("unknown sgr {}", item),
    }
}

#[derive(Debug)]
pub enum ClearMode {
    ToEnd,
    ToBeginning,
    Both,
}

impl From<usize> for ClearMode {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::ToEnd,
            1 => Self::ToBeginning,
            _ => Self::Both,
        }
    }
}

#[derive(Debug)]
pub enum AnsiOutput {
    Text(Vec<u8>),
    Backspace,
    ClearToEndOfLine(ClearMode),
    ClearToEOS,
    ScrollDown,
    MoveCursor(usize, usize),
    Bell,
    Sgr(GraphicRendition),
    CursorUp(usize),
    CursorDown(usize),
    MoveCursorHorizontal(usize),
    HideCursor,
    ShowCursor,
    CursorBackward(usize),
    CursorForward(usize),
}

impl Ansi {
    pub fn new() -> Self {
        Self {
            state: AnsiState::Empty,
        }
    }

    pub fn push(&mut self, data: &[u8]) -> Vec<AnsiOutput> {
        let mut res = vec![];
        let mut text_output = Vec::new();

        for b in data {
            match &mut self.state {
                AnsiState::Empty => {
                    if *b == ansi_codes::ESC {
                        self.state = AnsiState::Escape;
                        continue;
                    }
                    if *b == ansi_codes::BS {
                        if !text_output.is_empty() {
                            res.push(AnsiOutput::Text(std::mem::take(&mut text_output)));
                        }
                        res.push(AnsiOutput::Backspace);
                        continue;
                    }
                    if *b == ansi_codes::BEL {
                        res.push(AnsiOutput::Bell);
                        continue;
                    }
                    text_output.push(*b);
                }
                AnsiState::Escape => {
                    if !text_output.is_empty() {
                        res.push(AnsiOutput::Text(std::mem::take(&mut text_output)));
                    }
                    match *b {
                        ansi_codes::ESC_START => {
                            self.state = AnsiState::Csi(CSIParser::new());
                        }
                        ansi_codes::SCROLL_REVERSE => {
                            res.push(AnsiOutput::ScrollDown);
                            self.state = AnsiState::Empty;
                        }
                        _ => {
                            println!("unknown ansi {b} {:#02x}", b);
                        }
                    }
                }
                AnsiState::Csi(parser) => match parser.push(*b) {
                    Some(Ok(d)) => {
                        #[allow(clippy::single_match)]
                        match d.func {
                            ansi_codes::SGR => {
                                let params = parse_params(&d.params);
                                if params.is_empty() {
                                    res.push(AnsiOutput::Sgr(GraphicRendition::BackgroundColor(
                                        Color::BLACK,
                                    )));
                                    res.push(AnsiOutput::Sgr(GraphicRendition::ForegroundColor(
                                        Color::WHITE,
                                    )));
                                } else {
                                    if params[0] == 0 {
                                        res.push(AnsiOutput::Sgr(
                                            GraphicRendition::BackgroundColor(Color::BLACK),
                                        ));
                                        res.push(AnsiOutput::Sgr(
                                            GraphicRendition::ForegroundColor(Color::WHITE),
                                        ));
                                    } else if params.len() >= 3 && params[0] == 38 && params[1] == 5
                                    {
                                        res.push(AnsiOutput::Sgr(color_8bit(params[2] as u8)));
                                    } else {
                                        for param in params {
                                            // TODO: ugly hack to only take the color for now until we
                                            // properly handle all the graphic rendition things, like
                                            // "bold" for example
                                            if param >= 30 && param <= 47 {
                                                res.push(AnsiOutput::Sgr((param as u8).into()));
                                            }
                                        }
                                    }
                                }
                            }
                            ansi_codes::CLEAR_LINE => {
                                let params = parse_params(&d.params);
                                let mode: usize = if params.is_empty() { 0 } else { params[0] };
                                res.push(AnsiOutput::ClearToEndOfLine(mode.into()));
                            }
                            ansi_codes::CLEAR_EOS => res.push(AnsiOutput::ClearToEOS),
                            ansi_codes::CURSOR_POSITION => {
                                let params = parse_params(&d.params);
                                let x = if params.len() <= 1 { 1 } else { params[1] };
                                let y = if params.is_empty() { 1 } else { params[0] };
                                res.push(AnsiOutput::MoveCursor(x - 1, y - 1));
                            }
                            ansi_codes::CURSOR_HORIZONTAL_POSITION => {
                                let params = parse_params(&d.params);
                                let x = if params.is_empty() { 1 } else { params[0] };
                                res.push(AnsiOutput::MoveCursorHorizontal(x));
                            }
                            ansi_codes::CURSOR_UP => {
                                let params = parse_params(&d.params);
                                let amount = if params.is_empty() { 1 } else { params[0] };
                                res.push(AnsiOutput::CursorUp(amount));
                            }
                            ansi_codes::CURSOR_DOWN => {
                                let params = parse_params(&d.params);
                                let amount = if params.is_empty() { 1 } else { params[0] };
                                res.push(AnsiOutput::CursorDown(amount));
                            }
                            ansi_codes::CURSOR_FORWARD => {
                                let params = parse_params(&d.params);
                                let amount = if params.is_empty() { 1 } else { params[0] };
                                res.push(AnsiOutput::CursorForward(amount));
                            }
                            ansi_codes::CURSOR_BACKWARD => {
                                let params = parse_params(&d.params);
                                let amount = if params.is_empty() { 1 } else { params[0] };
                                res.push(AnsiOutput::CursorBackward(amount));
                            }
                            ansi_codes::HIDE_CURSOR => res.push(AnsiOutput::HideCursor),
                            ansi_codes::SHOW_CURSOR => res.push(AnsiOutput::ShowCursor),
                            _ => {
                                println!("unknown func {} {}", d.func, d.func as char);
                            }
                        }
                        self.state = AnsiState::Empty;
                    }
                    Some(Err(e)) => {
                        println!("Gotta do something with this error {e:?}");
                    }
                    _ => {} // CSI not finished, nothing to do
                },
            }
        }

        if !text_output.is_empty() {
            res.push(AnsiOutput::Text(text_output));
        }

        res
    }
}

fn parse_params(params: &[u8]) -> Vec<usize> {
    if params.is_empty() {
        return vec![];
    }
    params
        .split(|v| *v == b';')
        .map(parse_usize_param)
        .collect()
}

fn parse_usize_param(param: &[u8]) -> usize {
    let str = std::str::from_utf8(param).expect("Shoud be a number");
    str.parse().map_or(0, |v| v)
}
