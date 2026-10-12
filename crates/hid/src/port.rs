use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::path::Path;

use arctis_core::protocol::Frame;
use rustix::fs::{Mode, OFlags};
use rustix::io::Errno;

const READ_CAPACITY: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenError {
    Denied,
    Gone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortGone;

#[derive(Debug)]
pub enum Received {
    Report(Vec<u8>),
    Empty,
}

pub trait Opener {
    fn open(&mut self, path: &Path) -> Result<Port, OpenError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HidrawOpener;

impl Opener for HidrawOpener {
    fn open(&mut self, path: &Path) -> Result<Port, OpenError> {
        Port::open(path)
    }
}

#[derive(Debug)]
pub struct Port {
    fd: OwnedFd,
}

impl Port {
    pub fn open(path: &Path) -> Result<Self, OpenError> {
        let flags = OFlags::RDWR | OFlags::NONBLOCK | OFlags::CLOEXEC | OFlags::NOCTTY;
        rustix::fs::open(path, flags, Mode::empty())
            .map(Self::from_fd)
            .map_err(classify)
    }

    #[must_use]
    pub const fn from_fd(fd: OwnedFd) -> Self {
        Self { fd }
    }

    pub fn write(&self, frame: &Frame) -> Result<(), PortGone> {
        retrying(|| rustix::io::write(&self.fd, frame.as_bytes()))
            .map(|_| ())
            .map_err(|_| PortGone)
    }

    pub fn read(&self) -> Result<Received, PortGone> {
        let mut buffer = [0; READ_CAPACITY];
        match retrying(|| rustix::io::read(&self.fd, &mut buffer)) {
            Ok(0) => Err(PortGone),
            Ok(length) => Ok(Received::Report(buffer[..length].to_vec())),
            Err(Errno::AGAIN) => Ok(Received::Empty),
            Err(_) => Err(PortGone),
        }
    }
}

impl AsFd for Port {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

#[must_use]
pub fn classify(errno: Errno) -> OpenError {
    match errno {
        Errno::ACCESS | Errno::PERM | Errno::NOENT => OpenError::Denied,
        _ => OpenError::Gone,
    }
}

fn retrying<T>(mut call: impl FnMut() -> rustix::io::Result<T>) -> rustix::io::Result<T> {
    loop {
        let result = call();
        if !matches!(result, Err(Errno::INTR)) {
            return result;
        }
    }
}
