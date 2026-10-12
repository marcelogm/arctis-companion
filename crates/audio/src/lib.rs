#![forbid(unsafe_code)]

pub mod changes;
pub mod eq_sink;
pub mod graph;
pub mod handle;
pub mod links;
pub mod outputs;
pub mod registry;
pub mod samples;
pub mod streams;
pub mod system_engine;
pub mod volume;
mod worker;

pub use handle::{AudioCommands, AudioHandle};

pub fn init() {
    pipewire::init();
}
