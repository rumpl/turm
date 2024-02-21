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

pub enum SelectGraphicsRendition {
    Reset,
    ForegroundBlack,
    ForegroundRed,
    ForegroundGreen,
    ForegroundYellow,
}

impl From<usize> for SelectGraphicsRendition {
    fn from(item: usize) -> Self {
        match item {
            30 => SelectGraphicsRendition::ForegroundBlack,
            31 => SelectGraphicsRendition::ForegroundRed,
            32 => SelectGraphicsRendition::ForegroundGreen,
            33 => SelectGraphicsRendition::ForegroundYellow,
            _ => SelectGraphicsRendition::Reset,
        }
    }
}

pub enum AnsiOutput {
    Text(Vec<u8>),
    SGR(SelectGraphicsRendition),
}

impl Ansi {
    pub fn new() -> Self {
        Self {
            state: AnsiState::Empty,
        }
    }

    pub fn push(&mut self, data: &[u8]) -> Vec<AnsiOutput> {
        let mut res = vec![];
        let mut text_output = vec![];

        for b in data {
            match &mut self.state {
                AnsiState::Empty => {
                    if *b == ansi_codes::ESC {
                        self.state = AnsiState::Escape;
                        continue;
                    }
                    text_output.push(*b);
                }
                AnsiState::Escape => match *b {
                    ansi_codes::ESC_START => {
                        self.state = AnsiState::CSI(CSIParser::new());
                    }
                    _ => {
                        println!("unknown ansi {b}");
                    }
                },
                AnsiState::CSI(parser) => {
                    println!("csi parse {:x} {}", b, *b as char);
                    match parser.push(*b) {
                        CSIParserState::Finished(d) => {
                            match d {
                                ansi_codes::SGR => {
                                    let color = parse_params(&parser.params);
                                    res.push(AnsiOutput::SGR(color.into()));
                                }
                                _ => {}
                            }
                            self.state = AnsiState::Empty;
                        }
                        _ => {}
                    }
                }
            }
        }

        if !text_output.is_empty() {
            res.push(AnsiOutput::Text(text_output));
        }

        res
    }
}

fn parse_params(params: &[u8]) -> usize {
    // TODO: this is not how params work, there can be more than one, separated by ";"
    // and they can be empty, which means ... 0?
    let str = std::str::from_utf8(params).expect("parameters should be valid utf8");
    str.parse().expect("should be a number")
}
