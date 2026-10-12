#![forbid(unsafe_code)]

pub mod graph;
pub mod links;
pub mod outputs;
pub mod streams;

pub fn init() {
    pipewire::init();
}
