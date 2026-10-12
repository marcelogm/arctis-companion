use crate::messages::audio::SoundPath;

pub const RULE_ENTER: u64 = 1_000;
pub const RULE_LEAVE: u64 = 10_000;
pub const DONGLE_SWITCH_INTERVAL: u64 = 30_000;
pub const BLUETOOTH_SWITCH_INTERVAL: u64 = 2_000;
pub const EDITOR_THROTTLE: u64 = 250;
pub const QUIT_GRACE: u64 = 1_000;
pub const CRITICAL_BATTERY: u8 = 5;
pub const BATTERY_HYSTERESIS: u8 = 3;

#[must_use]
pub const fn auto_switch_min_interval(path: SoundPath) -> u64 {
    match path {
        SoundPath::Dongle => DONGLE_SWITCH_INTERVAL,
        SoundPath::Bluetooth => BLUETOOTH_SWITCH_INTERVAL,
    }
}
