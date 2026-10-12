#![forbid(unsafe_code)]

pub mod changes;
pub mod graph;
pub mod links;
pub mod outputs;
pub mod streams;

pub fn init() {
    pipewire::init();
}
