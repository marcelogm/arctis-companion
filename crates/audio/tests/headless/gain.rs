use arctis_core::eq::response::{preamp_db_at, response};
use arctis_core::eq::{Curve, Filter, FilterKind, Gain, Hertz, Q};
use arctis_core::messages::audio::AudioCommand;

use crate::lab::{BLUEZ, Lab, flat};
use crate::meter::level_db;
use crate::tone::{TONE_HZ, Tone};

const TOLERANCE_DB: f64 = 0.5;

fn peak(hz: u16, half_db: i8, q_milli: u16) -> Filter {
    Filter::new(
        FilterKind::Peaking,
        Hertz::new(hz).expect("hz"),
        Gain::from_half_db(half_db).expect("gain"),
        Q::from_milli(q_milli).expect("q"),
    )
    .expect("filter")
}

fn shaped() -> Curve {
    Curve::new(vec![peak(1000, 12, 2000), peak(100, 18, 1000)]).expect("curve")
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn a_sine_at_1_khz_follows_the_response_minus_the_preamp() {
    let lab = Lab::open();
    lab.insert(flat());
    let _tone = Tone::play(Vec::new(), Vec::new());
    let reference = level_db(BLUEZ);
    lab.send(AudioCommand::Retarget(shaped()));
    let shaped_level = level_db(BLUEZ);
    let expected = f64::from(response(&shaped(), &[TONE_HZ])[0]) + preamp_db_at(&shaped(), 48_000);
    let measured = shaped_level - reference;
    assert!(
        (measured - expected).abs() <= TOLERANCE_DB,
        "measured {measured:.2} dB, expected {expected:.2} dB"
    );
}
