use crate::messages::audio::SoundPath;

pub const PATHS: [SoundPath; 2] = [SoundPath::Dongle, SoundPath::Bluetooth];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PerPath<T> {
    pub dongle: T,
    pub bluetooth: T,
}

impl<T> PerPath<T> {
    pub fn from_fn(mut value_for: impl FnMut(SoundPath) -> T) -> Self {
        Self {
            dongle: value_for(SoundPath::Dongle),
            bluetooth: value_for(SoundPath::Bluetooth),
        }
    }

    #[must_use]
    pub const fn get(&self, path: SoundPath) -> &T {
        match path {
            SoundPath::Dongle => &self.dongle,
            SoundPath::Bluetooth => &self.bluetooth,
        }
    }

    pub fn get_mut(&mut self, path: SoundPath) -> &mut T {
        match path {
            SoundPath::Dongle => &mut self.dongle,
            SoundPath::Bluetooth => &mut self.bluetooth,
        }
    }

    pub fn set(&mut self, path: SoundPath, value: T) {
        *self.get_mut(path) = value;
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        [&self.dongle, &self.bluetooth].into_iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.dongle, &mut self.bluetooth].into_iter()
    }
}
