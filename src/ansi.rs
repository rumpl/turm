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
pub enum SelectGraphicRendition {
    Reset,
    ForegroundBlack,
    ForegroundRed,
    ForegroundGreen,
    ForegroundYellow,
    ForegroundBlue,
    ForegroundMagenta,
    ForegroundCyan,
    ForegroundWhite,
    ForegroundGrey,
    ForegroundBrightRed,
    ForegroundBrightGreen,
    ForegroundBrightYellow,
    ForegroundBrightBlue,
    ForegroundBrightMagenta,
    ForegroundBrightCyan,
    ForegroundBrightWhite,
}

impl From<usize> for SelectGraphicRendition {
    fn from(item: usize) -> Self {
        match item {
            30 => Self::ForegroundBlack,
            31 => Self::ForegroundRed,
            32 => Self::ForegroundGreen,
            33 => Self::ForegroundYellow,
            34 => Self::ForegroundBlue,
            35 => Self::ForegroundMagenta,
            36 => Self::ForegroundCyan,
            37 => Self::ForegroundWhite,
            90 => Self::ForegroundGrey,
            91 => Self::ForegroundBrightRed,
            92 => Self::ForegroundBrightGreen,
            93 => Self::ForegroundBrightYellow,
            94 => Self::ForegroundBrightBlue,
            95 => Self::ForegroundBrightMagenta,
            96 => Self::ForegroundBrightCyan,
            97 => Self::ForegroundBrightWhite,
            _ => Self::Reset,
        }
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
    Sgr(SelectGraphicRendition),
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
                                    res.push(AnsiOutput::Sgr(SelectGraphicRendition::Reset));
                                }
                                for param in params {
                                    res.push(AnsiOutput::Sgr(param.into()));
                                }
                            }
                            ansi_codes::CLEAR_LINE => {
                                let params = parse_params(&d.params);
                                let mode: usize = if params.is_empty() { 0 } else { params[0] };
                                res.push(AnsiOutput::ClearToEndOfLine(mode.into()));
                            }
                            ansi_codes::CLEAR_EOS => res.push(AnsiOutput::ClearToEOS),
                            ansi_codes::HOME => {
                                let params = parse_params(&d.params);
                                let x = if params.len() <= 1 { 1 } else { params[1] };
                                let y = if params.is_empty() { 1 } else { params[0] };
                                res.push(AnsiOutput::MoveCursor(x - 1, y - 1));
                            }
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
