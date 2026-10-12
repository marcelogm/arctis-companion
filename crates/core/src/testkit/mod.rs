pub mod english;
mod fake_headset;
pub mod memory_repositories;
mod replies;
pub mod repository_contracts;
mod simulation;

pub use fake_headset::{Behaviour, Emission, FakeHeadset};
pub use simulation::{Flap, Simulation};
