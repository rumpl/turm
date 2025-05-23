use crate::color::Color;

mod ansi_codes;

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
            if self.state != CSIParserState::Parameters {
                return Some(Err(CSIParserError::InvalidCSI));
            }
            self.result.params.push(b);
        } else if is_csi_intermediate(b) {
            self.state = CSIParserState::Intermediates;
            self.result.intermediates.push(b);
        } else if is_csi_terminator(b) {
            self.result.func = b;
            return Some(Ok(self.result.clone()));
        }

        None
    }
}

struct OscParseResult {
    title: String,
}

#[derive(Debug, PartialEq)]
enum OscState {
    Empty,
    TitleStart,
    Title,
}

#[derive(Debug)]
enum OscParserError {}

#[derive(Debug)]
struct OscParser {
    title: Vec<char>,
    state: OscState,
}

impl OscParser {
    fn new() -> Self {
        Self {
            title: Vec::new(),
            state: OscState::Empty,
        }
    }

    fn push(&mut self, b: char) -> Option<Result<OscParseResult, OscParserError>> {
        let c = b as u8;

        if c == ansi_codes::STRING_TERMINATOR || c == ansi_codes::BEL || c == ansi_codes::OSC_END {
            return Some(Ok(OscParseResult {
                title: self.title.iter().collect(),
            }));
        }

        if self.state == OscState::Title {
            self.title.push(b);
        }

        if c == b'2' {
            self.state = OscState::TitleStart;
            return None;
        }

        if self.state == OscState::TitleStart {
            self.state = OscState::Title;
            return None;
        }

        None
    }
}

enum AnsiState {
    Empty,
    Escape,
    Hash,
    Csi(CSIParser),
    Osc(OscParser),
}

pub struct Ansi {
    state: AnsiState,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GraphicRendition {
    BackgroundColor(Color),
    ForegroundColor(Color),
    Bold,
    Reset,
    Underline,
    Italic,
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
            // TODO: Why would we get a panic for an unknown sgr 38 when we run nvim?
            _ => Self::ForegroundColor(Color::WHITE), // panic!("unknown sgr {}", item),
        }
    }
}

fn color_8bit(item: u8) -> Color {
    match item {
        0 => Color::BLACK,
        1 => Color::RED,
        2 => Color::GREEN,
        3 => Color::YELLOW,
        4 => Color::BLUE,
        5 => Color::MAGENTA,
        6 => Color::CYAN,
        7 => Color::WHITE,

        8 => Color::GRAY,
        9 => Color::RED,
        10 => Color::GREEN,
        11 => Color::YELLOW,
        12 => Color::BLUE,
        13 => Color::MAGENTA,
        14 => Color::CYAN,
        15 => Color::WHITE,

        16..=231 => Color::from_rgb(
            (item - 16) & 0b1110_0000,
            (item - 16) & 0b0001_1100,
            (item - 16) & 0b0000_0011,
        ),
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
    Text(Vec<char>),
    Title(String),
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
    FillWithE,
    NextLine,
    DeleteCharacters(usize),
}

impl Ansi {
    pub fn new() -> Self {
        Self {
            state: AnsiState::Empty,
        }
    }

    pub fn push(&mut self, data: &[char]) -> Vec<AnsiOutput> {
        let mut res = vec![];
        let mut text_output: Vec<char> = Vec::new();

        for b in data {
            match &mut self.state {
                AnsiState::Empty => {
                    if *b as u8 == ansi_codes::ESC {
                        self.state = AnsiState::Escape;
                        continue;
                    }
                    if *b as u8 == ansi_codes::BS {
                        if !text_output.is_empty() {
                            res.push(AnsiOutput::Text(text_output.clone()));
                            text_output.clear();
                        }
                        res.push(AnsiOutput::Backspace);
                        continue;
                    }
                    if *b as u8 == ansi_codes::BEL {
                        res.push(AnsiOutput::Bell);
                        continue;
                    }
                    text_output.push(*b);
                }
                AnsiState::Escape => {
                    if !text_output.is_empty() {
                        res.push(AnsiOutput::Text(text_output.clone()));
                        text_output.clear();
                    }
                    match *b as u8 {
                        ansi_codes::ESC_START => {
                            self.state = AnsiState::Csi(CSIParser::new());
                        }
                        ansi_codes::HASH => {
                            self.state = AnsiState::Hash;
                        }
                        ansi_codes::OSC_START => {
                            self.state = AnsiState::Osc(OscParser::new());
                        }
                        ansi_codes::SCROLL_REVERSE => {
                            res.push(AnsiOutput::ScrollDown);
                            self.state = AnsiState::Empty;
                        }
                        ansi_codes::CURSOR_DOWNWARD => {
                            res.push(AnsiOutput::CursorDown(1));
                            self.state = AnsiState::Empty;
                        }
                        ansi_codes::NEXT_LINE => {
                            res.push(AnsiOutput::NextLine);
                            self.state = AnsiState::Empty;
                        }
                        _ => {
                            let first = ((*b as u8) & 0b0111_0000) >> 4;
                            let second = (*b as u8) & 0b0000_1111;
                            println!("unknown ansi {} {:#02x} {first}/{second}", { *b }, *b as u8);
                            self.state = AnsiState::Empty;
                        }
                    }
                }
                AnsiState::Hash => {
                    match *b as u8 {
                        ansi_codes::FILL_WITH_E => {
                            res.push(AnsiOutput::FillWithE);
                        }
                        _ => {
                            println!("Unknown hash func {b}");
                        }
                    };

                    self.state = AnsiState::Empty;
                }
                AnsiState::Osc(parser) => {
                    if let Some(Ok(d)) = parser.push(*b) {
                        res.push(AnsiOutput::Title(d.title.chars().collect()));
                        self.state = AnsiState::Empty;
                    }
                }
                AnsiState::Csi(parser) => match parser.push(*b as u8) {
                    Some(Ok(d)) => {
                        #[allow(clippy::single_match)]
                        match d.func {
                            ansi_codes::SGR => {
                                let params = parse_params(&d.params);
                                if params.len() == 1 && params[0] == 0 {
                                    res.push(AnsiOutput::Sgr(GraphicRendition::Reset));
                                } else if params.len() == 1 && params[0] == 1 {
                                    res.push(AnsiOutput::Sgr(GraphicRendition::Bold));
                                } else if params.len() == 1 && params[0] == 4 {
                                    res.push(AnsiOutput::Sgr(GraphicRendition::Underline));
                                } else if params.len() == 1 && params[0] == 3 {
                                    res.push(AnsiOutput::Sgr(GraphicRendition::Italic));
                                } else if params.is_empty() || params[0] == 0 {
                                    res.push(AnsiOutput::Sgr(GraphicRendition::BackgroundColor(
                                        Color::BLACK,
                                    )));
                                    res.push(AnsiOutput::Sgr(GraphicRendition::ForegroundColor(
                                        Color::WHITE,
                                    )));
                                } else if params.len() >= 3 && params[0] == 38 && params[1] == 5 {
                                    res.push(AnsiOutput::Sgr(GraphicRendition::ForegroundColor(
                                        color_8bit(params[2] as u8),
                                    )));
                                } else if params.len() >= 3 && params[0] == 48 && params[1] == 5 {
                                    res.push(AnsiOutput::Sgr(GraphicRendition::BackgroundColor(
                                        color_8bit(params[2] as u8),
                                    )));
                                } else if params.len() >= 3 && params[0] == 38 && params[1] == 2 {
                                    res.push(AnsiOutput::Sgr(GraphicRendition::ForegroundColor(
                                        Color::from_rgb(
                                            params[2] as u8,
                                            params[3] as u8,
                                            params[4] as u8,
                                        ),
                                    )));
                                } else if params.len() >= 3 && params[0] == 48 && params[1] == 2 {
                                    res.push(AnsiOutput::Sgr(GraphicRendition::BackgroundColor(
                                        Color::from_rgb(
                                            params[2] as u8,
                                            params[3] as u8,
                                            params[4] as u8,
                                        ),
                                    )));
                                } else {
                                    for param in params {
                                        // TODO: ugly hack to only take the color for now until we
                                        // properly handle all the graphic rendition things, like
                                        // "bold" for example
                                        if (30..=47).contains(&param) {
                                            res.push(AnsiOutput::Sgr((param as u8).into()));
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
                            ansi_codes::CURSOR_POSITION | ansi_codes::HVP => {
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
                            ansi_codes::DELETE_CHARACTER => {
                                let params = parse_params(&d.params);
                                let amount = if params.is_empty() { 1 } else { params[0] };
                                res.push(AnsiOutput::DeleteCharacters(amount));
                            }
                            ansi_codes::DCS => {
                                // Just ignore DCS sequences for now
                                // These are used by nvim but don't need special handling
                            }
                            _ => {
                                let first = (d.func & 0b0111_0000) >> 4;
                                let second = d.func & 0b0000_1111;
                                println!(
                                    "unknown func {} {} {first}/{second}",
                                    d.func, d.func as char
                                );
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
