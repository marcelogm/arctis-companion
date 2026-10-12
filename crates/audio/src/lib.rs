#![forbid(unsafe_code)]

pub mod changes;
pub mod graph;
pub mod links;
pub mod outputs;
pub mod samples;
pub mod streams;
pub mod volume;

pub fn init() {
    pipewire::init();
}
