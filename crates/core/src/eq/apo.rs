use super::curve::Curve;
use super::decimal::{parse_thousandths, rounded_quotient};
use super::filter::{Filter, FilterKind, Gain, Hertz, Q};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Frequency,
    Gain,
    Q,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clamp {
    pub position: usize,
    pub field: Field,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub curve: Curve,
    pub clamped: Vec<Clamp>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApoError {
    NoFilters,
    TooManyFilters(usize),
    Malformed { line: usize },
    UnsupportedKind { line: usize },
}

pub fn parse(text: &str) -> Result<Import, ApoError> {
    let raw = filter_lines(text)
        .map(|(line, body)| RawFilter::parse(line, body))
        .collect::<Result<Vec<_>, _>>()?;
    if raw.is_empty() {
        return Err(ApoError::NoFilters);
    }
    let converted = raw
        .iter()
        .enumerate()
        .map(|(position, filter)| filter.to_envelope(position))
        .collect::<Result<Vec<_>, _>>()?;
    let (filters, clamps): (Vec<Filter>, Vec<Vec<Clamp>>) = converted.into_iter().unzip();
    let curve = Curve::new(filters).map_err(|_| ApoError::TooManyFilters(raw.len()))?;
    Ok(Import {
        curve,
        clamped: clamps.concat(),
    })
}

#[must_use]
pub const fn kind_name(kind: FilterKind) -> &'static str {
    match kind {
        FilterKind::Peaking => "PK",
        FilterKind::LowShelf => "LSC",
        FilterKind::HighShelf => "HSC",
        FilterKind::LowPass => "LPQ",
        FilterKind::HighPass => "HPQ",
    }
}

fn filter_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines().enumerate().filter_map(|(index, line)| {
        let rest = line.trim_start().strip_prefix("Filter")?;
        let body = rest.split_once(':').map_or("", |(_, body)| body);
        Some((index + 1, body))
    })
}

struct RawFilter {
    line: usize,
    enabled: bool,
    kind: FilterKind,
    hz: i64,
    gain: i64,
    q: i64,
}

impl RawFilter {
    fn parse(line: usize, body: &str) -> Result<Self, ApoError> {
        let tokens: Vec<&str> = body.split_whitespace().collect();
        let malformed = ApoError::Malformed { line };
        let enabled = state(tokens.first().copied()).ok_or(malformed)?;
        let kind = kind_named(tokens.get(1).copied()).ok_or(ApoError::UnsupportedKind { line })?;
        let hz = number_after(&tokens, "Fc").ok_or(malformed)?;
        let q = number_after(&tokens, "Q").ok_or(malformed)?;
        let gain = gain_of(&tokens, kind).ok_or(malformed)?;
        Ok(Self {
            line,
            enabled,
            kind,
            hz,
            gain,
            q,
        })
    }

    fn to_envelope(&self, position: usize) -> Result<(Filter, Vec<Clamp>), ApoError> {
        let hz = within(
            rounded_quotient(self.hz, 1_000),
            Hertz::MIN.into(),
            Hertz::MAX.into(),
        );
        let gain = self.half_db();
        let q = within(self.q, Q::MIN_MILLI.into(), Q::MAX_MILLI.into());
        let clamps = [
            (hz.1, Field::Frequency),
            (gain.1, Field::Gain),
            (q.1, Field::Q),
        ]
        .into_iter()
        .filter(|(clamped, _)| *clamped)
        .map(|(_, field)| Clamp { position, field })
        .collect();
        Ok((self.filter(hz.0, gain.0, q.0)?, clamps))
    }

    fn half_db(&self) -> (i64, bool) {
        let half_db = rounded_quotient(self.gain, 500);
        if self.kind.has_gain() {
            within(half_db, Gain::MIN_HALF_DB.into(), Gain::MAX_HALF_DB.into())
        } else {
            (0, half_db != 0)
        }
    }

    fn filter(&self, hz: i64, half_db: i64, milli_q: i64) -> Result<Filter, ApoError> {
        let malformed = ApoError::Malformed { line: self.line };
        let hz = u16::try_from(hz).map_err(|_| malformed)?;
        let half_db = i8::try_from(half_db).map_err(|_| malformed)?;
        let milli_q = u16::try_from(milli_q).map_err(|_| malformed)?;
        let filter = Filter::from_parts(self.kind, hz, half_db, milli_q).map_err(|_| malformed)?;
        Ok(filter.with_enabled(self.enabled))
    }
}

fn within(value: i64, min: i64, max: i64) -> (i64, bool) {
    let bounded = value.clamp(min, max);
    (bounded, bounded != value)
}

fn state(token: Option<&str>) -> Option<bool> {
    let token = token?;
    if token.eq_ignore_ascii_case("ON") {
        Some(true)
    } else if token.eq_ignore_ascii_case("OFF") {
        Some(false)
    } else {
        None
    }
}

fn kind_named(token: Option<&str>) -> Option<FilterKind> {
    let token = token?;
    FilterKind::ALL
        .into_iter()
        .find(|kind| kind_name(*kind).eq_ignore_ascii_case(token))
}

fn gain_of(tokens: &[&str], kind: FilterKind) -> Option<i64> {
    match value_after(tokens, "Gain") {
        Some(text) => parse_thousandths(text),
        None if !kind.has_gain() => Some(0),
        None => None,
    }
}

fn number_after(tokens: &[&str], keyword: &str) -> Option<i64> {
    value_after(tokens, keyword).and_then(parse_thousandths)
}

fn value_after<'a>(tokens: &[&'a str], keyword: &str) -> Option<&'a str> {
    let index = tokens
        .iter()
        .position(|token| token.eq_ignore_ascii_case(keyword))?;
    tokens.get(index + 1).copied()
}
