use super::curve::Curve;
use super::decimal::{parse_thousandths, render_thousandths, rounded_quotient, thousandths_of};
use super::filter::{Clamped, Filter, FilterKind, Gain, Hertz, Q};
use super::response::preamp_db;

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
    let (filters, clamps): (Vec<Filter>, Vec<Vec<Clamp>>) = raw
        .iter()
        .enumerate()
        .map(|(position, filter)| filter.to_envelope(position))
        .unzip();
    let curve = Curve::new(filters).map_err(|_| ApoError::TooManyFilters(raw.len()))?;
    Ok(Import {
        curve,
        clamped: clamps.concat(),
    })
}

#[must_use]
pub fn render(curve: &Curve) -> String {
    let preamp = format!("Preamp: {} dB\n", preamp_text(curve));
    curve
        .filters()
        .iter()
        .enumerate()
        .fold(preamp, |mut text, (index, filter)| {
            text.push_str(&filter_line(index + 1, *filter));
            text
        })
}

#[must_use]
pub const fn kind_code(kind: FilterKind) -> &'static str {
    match kind {
        FilterKind::Peaking => "PK",
        FilterKind::LowShelf => "LSC",
        FilterKind::HighShelf => "HSC",
        FilterKind::LowPass => "LPQ",
        FilterKind::HighPass => "HPQ",
    }
}

fn filter_line(number: usize, filter: Filter) -> String {
    let state = if filter.is_enabled() { "ON" } else { "OFF" };
    let kind = kind_code(filter.kind());
    let hz = filter.hz().get();
    let gain = if filter.kind().has_gain() {
        format!(" Gain {} dB", decibels(filter.gain()))
    } else {
        String::new()
    };
    let q = render_thousandths(i64::from(filter.q().milli()));
    format!("Filter {number}: {state} {kind} Fc {hz} Hz{gain} Q {q}\n")
}

fn decibels(gain: Gain) -> String {
    render_thousandths(i64::from(gain.half_db()) * Gain::MILLI_DB_PER_STEP)
}

fn preamp_text(curve: &Curve) -> String {
    render_thousandths(thousandths_of(preamp_db(curve)).unwrap_or(0))
}

fn filter_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| filter_body(line).map(|body| (index + 1, body)))
}

fn filter_body(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let head_length = line
        .find(|character: char| character.is_whitespace() || character == ':')
        .unwrap_or(line.len());
    let (head, rest) = line.split_at(head_length);
    head.eq_ignore_ascii_case("Filter")
        .then(|| rest.split_once(':').map_or("", |(_, body)| body))
}

struct RawFilter {
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
        let kind = parse_kind(tokens.get(1).copied()).ok_or(ApoError::UnsupportedKind { line })?;
        let hz = number_after(&tokens, "Fc").ok_or(malformed)?;
        let q = number_after(&tokens, "Q").ok_or(malformed)?;
        let gain = gain_of(&tokens, kind).ok_or(malformed)?;
        Ok(Self {
            enabled,
            kind,
            hz,
            gain,
            q,
        })
    }

    fn to_envelope(&self, position: usize) -> (Filter, Vec<Clamp>) {
        let hz = Hertz::clamped(rounded_quotient(self.hz, 1_000));
        let gain = self.bounded_gain();
        let q = Q::clamped_milli(self.q);
        let clamps = [
            (hz.clamped, Field::Frequency),
            (gain.clamped, Field::Gain),
            (q.clamped, Field::Q),
        ]
        .into_iter()
        .filter(|(clamped, _)| *clamped)
        .map(|(_, field)| Clamp { position, field })
        .collect();
        let filter = Filter::dropping_pass_gain(self.kind, hz.value, gain.value, q.value);
        (filter.with_enabled(self.enabled), clamps)
    }

    fn bounded_gain(&self) -> Clamped<Gain> {
        let gain = Gain::clamped_half_db(rounded_quotient(self.gain, Gain::MILLI_DB_PER_STEP));
        let dropped = !self.kind.has_gain() && gain.value != Gain::NONE;
        Clamped {
            clamped: gain.clamped || dropped,
            ..gain
        }
    }
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

fn parse_kind(token: Option<&str>) -> Option<FilterKind> {
    let token = token?;
    FilterKind::ALL
        .into_iter()
        .find(|kind| kind_code(*kind).eq_ignore_ascii_case(token))
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
