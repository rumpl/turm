use std::{
    ffi::CStr,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use ansi::Ansi;
use gui::TurmGui;
use nix::pty::ForkptyResult;
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

    match result {
        ForkptyResult::Parent { child, master } => {
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
                    let fd = master;

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
                                    let _ =
                                        nix::unistd::write(write_fd.try_clone().unwrap(), &text);
                                }
                                TerminalGuiInputMessage::ScrollUp(delta) => {
                                    let mut t = event_turm.lock().unwrap();
                                    t.scroll_up(delta, false);
                                }
                                TerminalGuiInputMessage::ScrollDown(delta) => {
                                    let mut t = event_turm.lock().unwrap();
                                    t.scroll_down(delta, false);
                                }
                            }
                        }
                    });

                    Ok(Box::<TurmGui>::new(TurmGui::new(
                        cc,
                        fd,
                        turm_arc,
                        TerminalGuiInput::new(rtx),
                        cols,
                        rows,
                    )))
                }),
            );
        }

        ForkptyResult::Child => {
            std::env::set_var("TERM", "turm");
            // std::env::set_var("TERMINFO", "/home/rumpl/dev/turm/res");
            std::env::set_var("TERMINFO", "/Users/rumpl/hack/turm/res");
            let command = CStr::from_bytes_with_nul(b"/bin/zsh\0").unwrap();
            let args = [command];
            let _ = nix::unistd::execvp(command, &args);
        }
    }
}
