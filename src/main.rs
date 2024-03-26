use std::{
    env,
    ffi::CStr,
    os::fd::{AsRawFd, OwnedFd},
};

use gui::TurmGui;

mod ansi;
mod ansi_codes;
mod cell;
mod grid;
mod gui;
mod row;
mod turm;

fn set_nonblock(fd: &OwnedFd) {
    let flags = nix::fcntl::fcntl(fd.as_raw_fd(), nix::fcntl::FcntlArg::F_GETFL).unwrap();
    let mut flags =
        nix::fcntl::OFlag::from_bits(flags & nix::fcntl::OFlag::O_ACCMODE.bits()).unwrap();
    flags.set(nix::fcntl::OFlag::O_NONBLOCK, true);

    nix::fcntl::fcntl(fd.as_raw_fd(), nix::fcntl::FcntlArg::F_SETFL(flags)).unwrap();
}

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

            set_nonblock(&result.master);
            let options = eframe::NativeOptions::default();
            _ = eframe::run_native(
                "💩 Turm 💩",
                options,
                Box::new(|cc| Box::<TurmGui>::new(TurmGui::new(cc, result.master))),
            );
        }
        nix::unistd::ForkResult::Child => {
            std::env::set_var("TERM", "turm");
            std::env::set_var("TERMINFO", "/home/rumpl/dev/turm/res");
            let command = CStr::from_bytes_with_nul(b"/bin/sh\0").unwrap();
            let args = [command];
            let _ = nix::unistd::execvp(command, &args);
        }
    }
}
