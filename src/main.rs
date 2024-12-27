use std::{
    ffi::CStr,
    os::fd::AsRawFd,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use ansi::Ansi;
use gui::TurmGui;
use terminal_gui_input::{TerminalGuiInput, TerminalGuiInputMessage};
use terminal_io::TerminalIO;
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

            let options = eframe::NativeOptions::default();
            _ = eframe::run_native(
                "💩 Turm 💩",
                options,
                Box::new(|cc| {
                    let fd = result.master;

                    let cols: usize = 92;
                    let rows: usize = 34;

                    let rs = cc.egui_ctx.clone();

                    let write_fd = fd.try_clone().unwrap();
                    let read_fd = fd.try_clone().unwrap();

                    let turm_arc = Arc::new(Mutex::new(Turm::new(cols, rows)));

                    let turm = turm_arc.clone();
                    // Thread that reads output from the shell and sends it to the gui
                    thread::spawn(move || {
                        let ansi = Ansi::new();
                        let mut terminal_io = TerminalIO::new(ansi, read_fd, turm);
                        terminal_io.start_io(|| {
                            rs.request_repaint();
                        });
                    });

                    let (rtx, rrx) = mpsc::channel::<TerminalGuiInputMessage>();
                    let event_turm = turm_arc.clone();
                    // Thread that gets user input and sends it to the shell
                    thread::spawn(move || loop {
                        if let Ok(input) = rrx.recv() {
                            match input {
                                TerminalGuiInputMessage::Text(text) => {
                                    let _ = nix::unistd::write(write_fd.as_raw_fd(), &text);
                                }
                                TerminalGuiInputMessage::ScrollUp => {
                                    let mut t = event_turm.lock().unwrap();
                                    t.scroll_up(false);
                                }
                                TerminalGuiInputMessage::ScrollDown => {
                                    let mut t = event_turm.lock().unwrap();
                                    t.scroll_down(false);
                                }
                            }
                        }
                    });

                    Box::<TurmGui>::new(TurmGui::new(
                        cc,
                        fd,
                        turm_arc,
                        TerminalGuiInput::new(rtx),
                        cols,
                        rows,
                    ))
                }),
            );
        }

        nix::unistd::ForkResult::Child => {
            std::env::set_var("TERM", "turm");
            std::env::set_var("TERMINFO", "/home/rumpl/dev/turm/res");
            let command = CStr::from_bytes_with_nul(b"/bin/bash\0").unwrap();
            let args = [command];
            let _ = nix::unistd::execvp(command, &args);
        }
    }
}
