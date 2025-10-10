pub const ESC: u8 = b'\x1b';
pub const BS: u8 = b'\x08';
pub const BEL: u8 = b'\x07';
pub const ESC_START: u8 = b'['; // CSI - Control Sequence Introducer
pub const HASH: u8 = b'#';
pub const OSC_START: u8 = b']'; // OSC - Operating System Command
pub const STRING_TERMINATOR: u8 = b'\\'; // ST - String Terminator (used after ESC)
pub const SCROLL_REVERSE: u8 = b'M'; // RI - Reverse Index

pub const FILL_WITH_E: u8 = b'8'; // DECALN - Screen Alignment Pattern

pub const SGR: u8 = b'm'; // Select Graphic Rendition
pub const CURSOR_UP: u8 = b'A'; // CUU - Cursor Up
pub const CURSOR_DOWN: u8 = b'B'; // CUD - Cursor Down
pub const CURSOR_FORWARD: u8 = b'C'; // CUF - Cursor Forward
pub const CURSOR_BACKWARD: u8 = b'D'; // CUB - Cursor Back
pub const HIDE_CURSOR: u8 = b'l'; // DECTCEM - Hide cursor (used with ?25)
pub const SHOW_CURSOR: u8 = b'h'; // DECTCEM - Show cursor (used with ?25)
pub const CLEAR_LINE: u8 = b'K'; // EL - Erase in Line
pub const CLEAR_EOS: u8 = b'J'; // ED - Erase in Display
pub const DELETE_CHARACTER: u8 = b'P'; // DCH - Delete Character
pub const CURSOR_POSITION: u8 = b'H'; // CUP - Cursor Position
pub const HVP: u8 = b'f'; // HVP - Horizontal and Vertical Position
pub const CURSOR_HORIZONTAL_POSITION: u8 = b'G'; // CHA - Cursor Horizontal Absolute
pub const DCS: u8 = b'p'; // Device Control String

pub const NEXT_LINE: u8 = b'E'; // CNL - Cursor Next Line
pub const CURSOR_DOWNWARD: u8 = b'D'; // Same as CURSOR_DOWN (CUD)
