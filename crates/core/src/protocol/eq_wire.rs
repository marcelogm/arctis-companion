use super::frame::DecodeError;
use crate::eq::curve::MAX_FILTERS;
use crate::eq::{Curve, EnvelopeError, Filter, FilterKind, Gain, Hertz, Q};

pub const BAND_LEN: usize = 6;
pub const BANDS_LEN: usize = BAND_LEN * MAX_FILTERS;
pub(crate) const FIRST_BAND: usize = 1;
const DISABLED_HZ: u16 = 20_001;
const DISABLED_MILLI_Q: u16 = 1000;
const FLAT_GAIN_BYTE: i8 = 20;
const DISABLED_BAND: [u8; BAND_LEN] = band(
    DISABLED_HZ,
    kind_code(FilterKind::Peaking),
    FLAT_GAIN_BYTE.cast_unsigned(),
    DISABLED_MILLI_Q,
);
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
        .get(FIRST_BAND..)
        .and_then(<[u8]>::first_chunk::<BANDS_LEN>)
        .ok_or(DecodeError::TooShort {
            needed: FIRST_BAND + BANDS_LEN,
            got: frame.len(),
        })?;
    let mut slots = [None; MAX_FILTERS];
    for (index, (slot, band)) in slots
        .iter_mut()
        .zip(bands.as_chunks::<BAND_LEN>().0)
        .enumerate()
    {
        *slot = decode_band(*band, FIRST_BAND + index * BAND_LEN)?;
    }
    Ok(Curve::from_slots(slots))
}

const fn band(hz: u16, kind: u8, gain: u8, milli_q: u16) -> [u8; BAND_LEN] {
    let [hz_low, hz_high] = hz.to_le_bytes();
    let [q_low, q_high] = milli_q.to_le_bytes();
    [hz_low, hz_high, kind, gain, q_low, q_high]
}

fn encode_filter(filter: Filter) -> [u8; BAND_LEN] {
    band(
        filter.hz().get(),
        kind_code(filter.kind()),
        gain_byte(filter.gain()),
        filter.q().milli(),
    )
}

fn decode_band(band: [u8; BAND_LEN], offset: usize) -> Result<Option<Filter>, DecodeError> {
    let [hz_low, hz_high, kind, gain, q_low, q_high] = band;
    let hz = u16::from_le_bytes([hz_low, hz_high]);
    if hz == DISABLED_HZ {
        return Ok(None);
    }
    let hz = at(Hertz::new(hz), offset)?;
    let kind = kind_from_code(kind).ok_or(DecodeError::InvalidValue { offset: offset + 2 })?;
    let gain = at(gain_from_byte(gain, kind), offset + 3)?;
    let q = at(
        Q::from_milli(u16::from_le_bytes([q_low, q_high])),
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
