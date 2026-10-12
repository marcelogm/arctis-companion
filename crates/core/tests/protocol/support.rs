use std::fs;
use std::path::{Path, PathBuf};

use arctis_core::eq::{Curve, Filter, FilterKind};
use arctis_core::protocol::firmware::Version;
use arctis_core::settings::headset_settings::HeadsetSettings;
use arctis_core::settings::values::{
    AutoPowerOff, MicVolume, MuteLight, Percent, Sidetone, WirelessMode,
};

pub const DISABLED_BAND: [u8; 6] = [0x21, 0x4e, 0x01, 0x14, 0xe8, 0x03];
const FACTORY_HZ: [u16; 10] = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

pub fn golden(name: &str) -> Vec<u8> {
    let text = fs::read_to_string(golden_dir().join(name)).expect("golden file is readable");
    text.split_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).expect("golden bytes are hex"))
        .collect()
}

pub fn filter(kind: FilterKind, hz: u16, half_db: i8, milli_q: u16) -> Filter {
    Filter::from_parts(kind, hz, half_db, milli_q).expect("filter in envelope")
}

pub fn factory_flat() -> Curve {
    let filters = FACTORY_HZ
        .iter()
        .map(|&hz| filter(FilterKind::Peaking, hz, 0, 1410))
        .collect();
    Curve::new(filters).expect("ten filters")
}

pub fn mic(level: u8) -> MicVolume {
    MicVolume::try_from(level).expect("mic volume")
}

pub fn sidetone(level: u8) -> Sidetone {
    Sidetone::try_from(level).expect("sidetone")
}

pub fn auto_off(minutes: u8) -> AutoPowerOff {
    AutoPowerOff::try_from(minutes).expect("auto power off choice")
}

pub fn percent(value: u8) -> Percent {
    Percent::try_from(value).expect("percent")
}

pub const fn version(major: u8, minor: u8, patch: u8) -> Version {
    Version {
        major,
        minor,
        patch,
    }
}

pub fn sample_settings() -> HeadsetSettings {
    HeadsetSettings {
        mic_volume: mic(15),
        sidetone: sidetone(0),
        mute_light: MuteLight::High,
        volume_limiter: true,
        wireless_mode: WirelessMode::Standard,
        auto_power_off: auto_off(30),
        equalizer: Curve::default(),
    }
}
