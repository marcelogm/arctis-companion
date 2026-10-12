use std::os::fd::BorrowedFd;
use std::time::Instant;

use arctis_core::time::Millis;
use rustix::event::{PollFd, PollFlags, Timespec};

pub trait Waiter {
    fn now(&self) -> Millis;

    fn wait(&mut self, fds: &[BorrowedFd<'_>], deadline: Option<Millis>) -> Vec<bool>;
}

#[derive(Clone, Copy, Debug)]
pub struct PollWaiter {
    epoch: Instant,
}

impl Default for PollWaiter {
    fn default() -> Self {
        Self::new()
    }
}

impl PollWaiter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }
}

impl Waiter for PollWaiter {
    fn now(&self) -> Millis {
        let elapsed = self.epoch.elapsed().as_millis();
        Millis(u64::try_from(elapsed).unwrap_or(u64::MAX))
    }

    fn wait(&mut self, fds: &[BorrowedFd<'_>], deadline: Option<Millis>) -> Vec<bool> {
        let mut polled: Vec<PollFd<'_>> = fds
            .iter()
            .map(|fd| PollFd::from_borrowed_fd(*fd, PollFlags::IN))
            .collect();
        let timeout = deadline.map(|at| timespec(at.since(self.now())));
        match rustix::event::poll(&mut polled, timeout.as_ref()) {
            Ok(_) => polled.iter().map(is_ready).collect(),
            Err(_) => vec![false; fds.len()],
        }
    }
}

fn is_ready(fd: &PollFd<'_>) -> bool {
    fd.revents()
        .intersects(PollFlags::IN | PollFlags::HUP | PollFlags::ERR)
}

fn timespec(millis: u64) -> Timespec {
    Timespec {
        tv_sec: i64::try_from(millis / 1_000).unwrap_or(i64::MAX),
        tv_nsec: i64::try_from(millis % 1_000 * 1_000_000).unwrap_or(0),
    }
}
