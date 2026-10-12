use super::frame::DecodeError;
use crate::eq::curve::MAX_FILTERS;
use crate::eq::{Curve, EnvelopeError, Filter, FilterKind, Gain, Hertz, Q};

pub const BAND_LEN: usize = 6;
pub const BANDS_LEN: usize = BAND_LEN * MAX_FILTERS;
const FIRST_BAND: usize = 1;
const DISABLED_HZ: u16 = 20_001;
const DISABLED_BAND: [u8; BAND_LEN] = [0x21, 0x4e, 0x01, 0x14, 0xe8, 0x03];
const FLAT_GAIN_BYTE: i8 = 20;
const NO_GAIN: Gain = match Gain::from_half_db(0) {
    Ok(gain) => gain,
    Err(_) => panic!("a flat gain is inside the envelope"),
};

#[must_use]
pub fn encode_curve(curve: &Curve) -> [u8; BANDS_LEN] {
    let mut bytes = [0; BANDS_LEN];
    let (bands, _) = bytes.as_chunks_mut::<BAND_LEN>();
    let mut enabled = curve
        .filters()
        .iter()
        .copied()
        .filter(|filter| filter.is_enabled());
    for band in bands {
        *band = enabled.next().map_or(DISABLED_BAND, encode_filter);
    }
    bytes
}

pub fn decode_curve(frame: &[u8]) -> Result<Curve, DecodeError> {
    let bands = frame
        .get(FIRST_BAND..FIRST_BAND + BANDS_LEN)
        .ok_or(DecodeError::TooShort {
            needed: FIRST_BAND + BANDS_LEN,
            got: frame.len(),
        })?;
    let (bands, _) = bands.as_chunks::<BAND_LEN>();
    let mut filters = Vec::with_capacity(MAX_FILTERS);
    for (slot, band) in bands.iter().enumerate() {
        filters.extend(decode_band(*band, FIRST_BAND + slot * BAND_LEN)?);
    }
    at(Curve::new(filters), FIRST_BAND)
}

fn encode_filter(filter: Filter) -> [u8; BAND_LEN] {
    let [hz_low, hz_high] = filter.hz().get().to_le_bytes();
    let [q_low, q_high] = filter.q().milli().to_le_bytes();
    [
        hz_low,
        hz_high,
        kind_code(filter.kind()),
        gain_byte(filter.gain()),
        q_low,
        q_high,
    ]
}

fn decode_band(band: [u8; BAND_LEN], offset: usize) -> Result<Option<Filter>, DecodeError> {
    let hz = u16::from_le_bytes([band[0], band[1]]);
    if hz == DISABLED_HZ {
        return Ok(None);
    }
    let hz = at(Hertz::new(hz), offset)?;
    let kind = kind_from_code(band[2]).ok_or(DecodeError::InvalidValue { offset: offset + 2 })?;
    let gain = at(gain_from_byte(band[3], kind), offset + 3)?;
    let q = at(
        Q::from_milli(u16::from_le_bytes([band[4], band[5]])),
        offset + 4,
    )?;
    at(Filter::new(kind, hz, gain, q), offset).map(Some)
}

fn at<T>(value: Result<T, EnvelopeError>, offset: usize) -> Result<T, DecodeError> {
    value.map_err(|_| DecodeError::InvalidValue { offset })
}

const fn gain_byte(gain: Gain) -> u8 {
    (gain.half_db() + FLAT_GAIN_BYTE).cast_unsigned()
}

fn gain_from_byte(byte: u8, kind: FilterKind) -> Result<Gain, EnvelopeError> {
    let half_db = i16::from(byte) - i16::from(FLAT_GAIN_BYTE);
    let gain = Gain::from_half_db(i8::try_from(half_db).map_err(|_| EnvelopeError::Gain)?)?;
    Ok(if kind.has_gain() { gain } else { NO_GAIN })
}

const fn kind_code(kind: FilterKind) -> u8 {
    match kind {
        FilterKind::Peaking => 1,
        FilterKind::LowPass => 2,
        FilterKind::HighPass => 3,
        FilterKind::LowShelf => 4,
        FilterKind::HighShelf => 5,
    }
}

fn kind_from_code(code: u8) -> Option<FilterKind> {
    FilterKind::ALL
        .into_iter()
        .find(|kind| kind_code(*kind) == code)
}
