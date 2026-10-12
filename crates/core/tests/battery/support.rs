use arctis_core::battery::sample::{Reading, Sample, UnixSeconds};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::values::Percent;

pub fn reading(percent: u8, charging: Charging, link: Link) -> Reading {
    Reading {
        percent: Percent::try_from(percent).expect("percent"),
        charging,
        link,
    }
}

pub fn discharging(percent: u8) -> Reading {
    reading(percent, Charging::Discharging, Link::Connected)
}

pub fn sample(at: u64, reading: Reading) -> Sample {
    Sample {
        at: UnixSeconds(at),
        reading,
    }
}
