use crate::ansi_codes;

fn is_csi_terminator(b: u8) -> bool {
    (0x40..=0x7e).contains(&b)
}

fn is_csi_param(b: u8) -> bool {
    (0x30..=0x3f).contains(&b)
}

#[derive(Clone, Copy)]
enum CSIParserState {
    Parameters,
    Intermediates,
    Finished(u8),
    Invalid,
}

struct CSIParser {
    params: Vec<u8>,
    state: CSIParserState,
}

impl CSIParser {
    fn new() -> Self {
        Self {
            state: CSIParserState::Parameters,
            params: vec![],
        }
    }

    fn push(&mut self, b: u8) -> CSIParserState {
        match &mut self.state {
            CSIParserState::Parameters => {
                if is_csi_terminator(b) {
                    self.state = CSIParserState::Finished(b);
                } else if is_csi_param(b) {
                    self.params.push(b);
                }
            }
            CSIParserState::Intermediates => {}
            CSIParserState::Finished(_) => {}
            CSIParserState::Invalid => {}
        }
        self.state
    }
}

enum AnsiState {
    Empty,
    Escape,
    CSI(CSIParser),
}

pub struct Ansi {
    state: AnsiState,
}

#[derive(Debug, Clone, Copy)]
pub enum SelectGraphicRendition {
    Reset,
    ForegroundBlack,
    ForegroundRed,
    ForegroundGreen,
    ForegroundYellow,
}

impl From<usize> for SelectGraphicRendition {
    fn from(item: usize) -> Self {
        match item {
            30 => SelectGraphicRendition::ForegroundBlack,
            31 => SelectGraphicRendition::ForegroundRed,
            32 => SelectGraphicRendition::ForegroundGreen,
            33 => SelectGraphicRendition::ForegroundYellow,
            _ => SelectGraphicRendition::Reset,
        }
    }
}

#[derive(Debug)]
pub enum AnsiOutput {
    Text(Vec<u8>),
    SGR(SelectGraphicRendition),
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
                    text_output.push(*b);
                }
                AnsiState::Escape => {
                    if !text_output.is_empty() {
                        res.push(AnsiOutput::Text(std::mem::take(&mut text_output)));
                    }
                    match *b {
                        ansi_codes::ESC_START => {
                            self.state = AnsiState::CSI(CSIParser::new());
                        }
                        _ => {
                            println!("unknown ansi {b}");
                        }
                    }
                }
                AnsiState::CSI(parser) => match parser.push(*b) {
                    CSIParserState::Finished(d) => {
                        match d {
                            ansi_codes::SGR => {
                                let params = parse_params(&parser.params);
                                for param in params {
                                    res.push(AnsiOutput::SGR(param.into()));
                                }
                            }
                            _ => {}
                        }
                        self.state = AnsiState::Empty;
                    }
                    _ => {}
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
    params
        .split(|v| *v == b';')
        .map(parse_usize_param)
        .collect()
}

fn parse_usize_param(param: &[u8]) -> usize {
    let str = std::str::from_utf8(param).expect("Shoud be a number");
    str.parse().map_or(0, |v| v)
}
