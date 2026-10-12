use std::os::fd::{BorrowedFd, OwnedFd};

use arctis_hid::waiter::poll_ready;

use rustix::net::{AddressFamily, SocketFlags, SocketType};

pub fn pair() -> (OwnedFd, OwnedFd) {
    rustix::net::socketpair(
        AddressFamily::UNIX,
        SocketType::SEQPACKET,
        SocketFlags::NONBLOCK | SocketFlags::CLOEXEC,
        None,
    )
    .expect("a socket pair")
}

pub fn send(fd: &OwnedFd, bytes: &[u8]) {
    rustix::io::write(fd, bytes).expect("a frame sent");
}

pub fn receive(fd: &OwnedFd) -> Option<Vec<u8>> {
    let mut buffer = [0; 256];
    match rustix::io::read(fd, &mut buffer) {
        Ok(length) if length > 0 => Some(buffer[..length].to_vec()),
        _ => None,
    }
}

pub fn poll_now(fds: &[BorrowedFd<'_>]) -> Vec<bool> {
    poll_ready(fds, Some(0))
}
