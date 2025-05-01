pub mod egui;
pub mod gtk4;

use crate::turm::Turm;
use std::{os::fd::OwnedFd, sync::Arc, sync::Mutex};

/// GuiTrait defines the interface for different GUI implementations
pub trait Gui {
    /// Create a new GUI instance
    fn new(fd: OwnedFd, turm: Arc<Mutex<Turm>>, cols: usize, rows: usize) -> Self
    where
        Self: Sized;

    /// Start the GUI application
    fn run(self)
    where
        Self: Sized;
}

/// Helper function to resize a terminal
pub fn resize(fd: impl std::os::fd::AsRawFd, cols: usize, rows: usize, font_size: f32, width: f32) {
    use nix::pty::Winsize;
    use std::io::Error;

    let ws = Winsize {
        ws_col: cols as u16,
        ws_row: rows as u16,
        ws_xpixel: cols as u16 * width as u16,
        ws_ypixel: rows as u16 * font_size as u16,
    };

    let res = unsafe { libc::ioctl(fd.as_raw_fd(), libc::TIOCSWINSZ, &ws as *const _) };
    if res < 0 {
        println!("ioctl TIOCSWINSZ failed: {}", Error::last_os_error());
        std::process::exit(1);
    }
}
