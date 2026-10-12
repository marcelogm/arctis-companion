#![forbid(unsafe_code)]

pub mod graph;
pub mod links;
pub mod outputs;

pub fn init() {
    pipewire::init();
}
