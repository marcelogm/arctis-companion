use super::curve::Curve;
use super::filter::{Filter, FilterKind};
use super::preset::{Preset, PresetId};

pub const FLAT_ID: &str = "flat";

const FACTORY_Q: u16 = 1_410;
const SHELF_Q: u16 = 707;

const fn band(kind: FilterKind, hz: u16, half_db: i8, milli_q: u16) -> Filter {
    let Ok(filter) = Filter::from_parts(kind, hz, half_db, milli_q) else {
        panic!("a built-in filter lies outside the envelope");
    };
    filter
}

const fn peaking(hz: u16, half_db: i8, milli_q: u16) -> Filter {
    band(FilterKind::Peaking, hz, half_db, milli_q)
}

const fn low_shelf(hz: u16, half_db: i8) -> Filter {
    band(FilterKind::LowShelf, hz, half_db, SHELF_Q)
}

const fn high_shelf(hz: u16, half_db: i8) -> Filter {
    band(FilterKind::HighShelf, hz, half_db, SHELF_Q)
}

const FLAT: [Filter; 10] = [
    peaking(32, 0, FACTORY_Q),
    peaking(64, 0, FACTORY_Q),
    peaking(125, 0, FACTORY_Q),
    peaking(250, 0, FACTORY_Q),
    peaking(500, 0, FACTORY_Q),
    peaking(1_000, 0, FACTORY_Q),
    peaking(2_000, 0, FACTORY_Q),
    peaking(4_000, 0, FACTORY_Q),
    peaking(8_000, 0, FACTORY_Q),
    peaking(16_000, 0, FACTORY_Q),
];
const BASS_BOOST: [Filter; 1] = [low_shelf(105, 12)];
const BASS_REDUCTION: [Filter; 1] = [low_shelf(120, -12)];
const TREBLE_SOFTEN: [Filter; 2] = [peaking(3_500, -3, 1_410), high_shelf(6_000, -9)];
const VOCAL_CLARITY: [Filter; 4] = [
    low_shelf(100, -6),
    peaking(250, -4, 1_000),
    peaking(2_500, 6, 1_000),
    peaking(5_000, 3, 1_410),
];
const FOOTSTEPS: [Filter; 4] = [
    low_shelf(150, -8),
    peaking(2_000, 8, 1_000),
    peaking(4_500, 10, 1_410),
    high_shelf(10_000, -4),
];
const V_SHAPE: [Filter; 3] = [
    low_shelf(100, 10),
    peaking(1_000, -6, 700),
    high_shelf(8_000, 9),
];
const MOVIE_NIGHT: [Filter; 3] = [
    low_shelf(80, 8),
    peaking(2_500, 4, 1_000),
    high_shelf(10_000, 3),
];
const PODCAST: [Filter; 4] = [
    low_shelf(90, -8),
    peaking(180, -4, 1_000),
    peaking(3_000, 6, 1_200),
    high_shelf(9_000, -4),
];
const LATE_NIGHT: [Filter; 3] = [
    low_shelf(120, -6),
    peaking(3_000, -2, 1_000),
    high_shelf(7_000, -6),
];

#[must_use]
pub fn flat() -> Preset {
    built_in(FLAT_ID, "Flat", FLAT)
}

#[must_use]
pub fn library() -> Vec<Preset> {
    vec![
        flat(),
        built_in("bass-boost", "Bass Boost", BASS_BOOST),
        built_in("bass-reduction", "Bass Reduction", BASS_REDUCTION),
        built_in("treble-soften", "Treble Soften", TREBLE_SOFTEN),
        built_in("vocal-clarity", "Vocal Clarity", VOCAL_CLARITY),
        built_in("footsteps", "Footsteps", FOOTSTEPS),
        built_in("v-shape", "V-Shape", V_SHAPE),
        built_in("movie-night", "Movie Night", MOVIE_NIGHT),
        built_in("podcast", "Podcast", PODCAST),
        built_in("late-night", "Late Night", LATE_NIGHT),
    ]
}

#[must_use]
pub fn find(id: &PresetId) -> Option<Preset> {
    library().into_iter().find(|preset| preset.id() == id)
}

fn built_in<const N: usize>(id: &str, name: &str, filters: [Filter; N]) -> Preset {
    Preset::built_in(id, name, Curve::from_array(filters))
}
