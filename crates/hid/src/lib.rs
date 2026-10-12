#![forbid(unsafe_code)]

pub mod discovery;
mod uevent;

pub use discovery::SysfsRoot;
