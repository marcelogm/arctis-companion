use arctis_core::eq::{Gain, Hertz};
use arctis_core::settings::values::Percent;

pub fn percent(value: u8) -> Percent {
    Percent::try_from(value).expect("percent")
}

pub fn gain(half_db: i8) -> Gain {
    Gain::from_half_db(half_db).expect("gain in the envelope")
}

pub fn hertz(hz: u16) -> Hertz {
    Hertz::new(hz).expect("frequency in the envelope")
}
