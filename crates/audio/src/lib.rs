#![forbid(unsafe_code)]

pub mod changes;
pub mod designer;
pub mod eq_names;
mod eq_sink;
pub mod graph;
pub mod handle;
pub mod insertion;
pub mod keys;
pub mod links;
pub mod metadata_value;
pub mod pod;
mod registry;
pub mod samples;
pub mod sinks;
pub mod streams;
mod system_engine;
pub mod volume;
pub mod wired;
mod worker;

pub use handle::{AudioCommands, AudioHandle};

pub fn init() {
    pipewire::init();
}
