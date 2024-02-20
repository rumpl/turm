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

fn is_csi_terminator(b: u8) -> bool {
    (0x40..=0x7e).contains(&b)
}

fn is_csi_param(b: u8) -> bool {
    (0x30..=0x3f).contains(&b)
}

impl CSIParser {
    fn new() -> Self {
        Self {
            state: CSIParserState::Parameters,
            params: vec![],
        }
    }

    fn push(&mut self, b: u8) {
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

pub enum AnsiOutput {
    Text(Vec<u8>),
    Color(usize),
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
                    if *b == b'\x1b' {
                        self.state = AnsiState::Escape;
                        continue;
                    }
                    text_output.push(*b);
                }
                AnsiState::Escape => match b {
                    b'[' => {
                        self.state = AnsiState::CSI(CSIParser::new());
                    }
                    _ => {
                        println!("ugh");
                    }
                },
                AnsiState::CSI(parser) => {
                    parser.push(*b);
                    println!("csi parse {:x} {}", b, *b as char);
                    match parser.state {
                        CSIParserState::Finished(d) => {
                            match d {
                                b'm' => {
                                    let color = parse_params(&parser.params);
                                    res.push(AnsiOutput::Color(color));
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
    let str = std::str::from_utf8(params).expect("parameters should be valid utf8");
    str.parse().expect("should be a number")
}
