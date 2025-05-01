use std::sync::{Arc, Mutex};

use gui::egui::EguiImpl;
use gui::Gui;
use turm::Turm;

mod ansi;
mod color;
mod font;
mod grid;
mod gui;
mod terminal_gui_input;
mod terminal_io;
mod turm;

fn main() {
    let result = unsafe { nix::pty::forkpty(None, None).unwrap() };

    match result.fork_result {
        nix::unistd::ForkResult::Parent { child } => {
            std::thread::spawn(move || {
                let Ok(res) = nix::sys::wait::waitpid(child, None) else {
                    std::process::exit(-1);
                };
                match res {
                    nix::sys::wait::WaitStatus::Exited(_, code) => std::process::exit(code),
                    _ => std::process::exit(-1),
                }
            });

            let fd = result.master;

            let cols: usize = 92;
            let rows: usize = 34;

            let turm_arc = Arc::new(Mutex::new(Turm::new(cols, rows)));

            // Create and run the GUI implementation
            let gui = EguiImpl::new(fd, turm_arc, cols, rows);
            gui.run();
        }

        nix::unistd::ForkResult::Child => {
            std::env::set_var("TERM", "turm");
            std::env::set_var("TERMINFO", "/home/rumpl/dev/turm/res");
            let command = c"/bin/bash";
            let args = [command];
            let _ = nix::unistd::execvp(command, &args);
        }
    }
}
