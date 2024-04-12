use std::{
    ops::DerefMut,
    os::fd::{AsRawFd, OwnedFd},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{ansi::Ansi, turm::Turm};

unsafe fn set_nonblocking(fd: i32) {
    use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};
    let _ = fcntl(fd, F_SETFL, fcntl(fd, F_GETFL, 0) | O_NONBLOCK);
}

pub struct TerminalIO {
    ansi: Ansi,
    fd: OwnedFd,
    turm: Arc<Mutex<Turm>>,
}

impl TerminalIO {
    pub fn new(ansi: Ansi, fd: OwnedFd, turm: Arc<Mutex<Turm>>) -> Self {
        Self { ansi, fd, turm }
    }

    pub fn start_io<F>(&mut self, repaint: F)
    where
        F: Fn(),
    {
        unsafe {
            set_nonblocking(self.fd.as_raw_fd());
        }
        let poller = polling::Poller::new().unwrap();
        unsafe {
            poller
                .add(self.fd.as_raw_fd(), polling::Event::readable(7))
                .unwrap();
        }

        let mut events = polling::Events::new();
        let timeout = Duration::new(0, 10_000_000); // 10ms

        loop {
            let mut buf = vec![0u8; 1024];

            events.clear();

            poller.wait(&mut events, Some(timeout)).unwrap();

            // For some reason on MacOS this thing only returns data on the first read,
            // resulting in messages being sent in 1024 byte chunks, this makes redraw slow on
            // heavy UI applications like neovim, this works fine on Linux though.
            let mut turm1 = self.turm.lock().unwrap();
            let turm = turm1.deref_mut();

            for _ in events.iter() {
                for _ in 0..30 {
                    thread::sleep(Duration::new(0, 1_000));

                    let ret = nix::unistd::read(self.fd.as_raw_fd(), &mut buf);
                    if let Ok(s) = ret {
                        if s != 0 {
                            let ansi_res = self.ansi.push(&buf[0..s]);
                            turm.parse(ansi_res);
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                repaint();
            }

            drop(turm1);

            poller
                .modify(&self.fd, polling::Event::readable(7))
                .unwrap();
        }
    }
}
