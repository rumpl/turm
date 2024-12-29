macro_rules! ANSI_CODE {
    ($a:expr, $b:expr) => {
        $b | $a << 4
    };
}

pub const ESC: u8 = b'\x1b';
pub const BS: u8 = b'\x08';
pub const BEL: u8 = b'\x07';
pub const ESC_START: u8 = b'[';
pub const HASH: u8 = ANSI_CODE!(2, 3); // # in normal
pub const OSC_START: u8 = ANSI_CODE!(5, 13);
pub const OSC_END: u8 = ANSI_CODE!(5, 12);
pub const SCROLL_REVERSE: u8 = b'M';

pub const FILL_WITH_E: u8 = b'8';

pub const STRING_TERMINATOR: u8 = ANSI_CODE!(9, 12);
pub const SGR: u8 = b'm';
pub const CURSOR_UP: u8 = b'A';
pub const CURSOR_DOWN: u8 = b'B';
pub const CURSOR_FORWARD: u8 = b'C';
pub const CURSOR_BACKWARD: u8 = b'D';
pub const HIDE_CURSOR: u8 = b'l';
pub const SHOW_CURSOR: u8 = b'h';
pub const CLEAR_LINE: u8 = b'K';
pub const CLEAR_EOS: u8 = b'J';
pub const DELETE_CHARACTER: u8 = b'P';
pub const CURSOR_POSITION: u8 = b'H';
pub const HVP: u8 = ANSI_CODE!(6, 6);
pub const CURSOR_HORIZONTAL_POSITION: u8 = b'G';

pub const NEXT_LINE: u8 = b'E';
pub const CURSOR_DOWNWARD: u8 = b'D';
