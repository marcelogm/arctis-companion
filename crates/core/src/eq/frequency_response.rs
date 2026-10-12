use super::curve::Curve;
use super::response::gains_db;

pub const HEADER: &str = "freq_hz,spl_db";
pub const LEVEL_LOW_HZ: f64 = 300.0;
pub const LEVEL_HIGH_HZ: f64 = 3_000.0;
const LEVEL_PROBES: u16 = 80;
const MIN_POINTS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub hz: f64,
    pub db: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    Header,
    Malformed { line: usize },
    NotAscending { line: usize },
    TooFewPoints,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrequencyResponse {
    points: Vec<Point>,
}

impl FrequencyResponse {
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let mut lines = (1..).zip(text.lines().map(str::trim_end));
        if lines.next().map(|(_, header)| header) != Some(HEADER) {
            return Err(ParseError::Header);
        }
        let points = lines
            .filter(|(_, line)| !line.is_empty())
            .map(|(number, line)| numbered_point(number, line))
            .collect::<Result<Vec<_>, _>>()?;
        ascending(&points)?;
        if points.len() < MIN_POINTS {
            return Err(ParseError::TooFewPoints);
        }
        Ok(Self {
            points: points.into_iter().map(|(_, point)| point).collect(),
        })
    }

    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    #[must_use]
    pub fn db_at(&self, hz: f64) -> f64 {
        let after = self.points.partition_point(|point| point.hz < hz);
        match (after.checked_sub(1), self.points.get(after)) {
            (Some(below), Some(&above)) => between(self.points[below], above, hz),
            (None, Some(first)) => first.db,
            (_, None) => self.points.last().map_or(0.0, |last| last.db),
        }
    }

    #[must_use]
    pub fn level_db(&self) -> f64 {
        let span = LEVEL_HIGH_HZ / LEVEL_LOW_HZ;
        let last = f64::from(LEVEL_PROBES - 1);
        let total: f64 = (0..LEVEL_PROBES)
            .map(|step| LEVEL_LOW_HZ * span.powf(f64::from(step) / last))
            .map(|hz| self.db_at(hz))
            .sum();
        total / f64::from(LEVEL_PROBES)
    }

    #[must_use]
    pub fn normalised(&self) -> Self {
        let level = self.level_db();
        self.with_levels(self.points.iter().map(|point| point.db - level))
    }

    #[must_use]
    pub fn equalized(&self, curve: &Curve) -> Self {
        let frequencies: Vec<f64> = self.points.iter().map(|point| point.hz).collect();
        let gains = gains_db(curve, &frequencies);
        self.with_levels(
            self.points
                .iter()
                .zip(gains)
                .map(|(point, gain)| point.db + gain),
        )
    }

    fn with_levels(&self, levels: impl Iterator<Item = f64>) -> Self {
        Self {
            points: self
                .points
                .iter()
                .zip(levels)
                .map(|(point, db)| Point { hz: point.hz, db })
                .collect(),
        }
    }
}

fn numbered_point(number: usize, line: &str) -> Result<(usize, Point), ParseError> {
    point(line)
        .map(|point| (number, point))
        .ok_or(ParseError::Malformed { line: number })
}

fn point(line: &str) -> Option<Point> {
    let (hz, db) = line.split_once(',')?;
    let (hz, db): (f64, f64) = (hz.trim().parse().ok()?, db.trim().parse().ok()?);
    (hz.is_finite() && hz > 0.0 && db.is_finite()).then_some(Point { hz, db })
}

fn ascending(points: &[(usize, Point)]) -> Result<(), ParseError> {
    match points.windows(2).find(|pair| pair[1].1.hz <= pair[0].1.hz) {
        Some(pair) => Err(ParseError::NotAscending { line: pair[1].0 }),
        None => Ok(()),
    }
}

fn between(below: Point, above: Point, hz: f64) -> f64 {
    let fraction = (hz / below.hz).ln() / (above.hz / below.hz).ln();
    below.db + fraction * (above.db - below.db)
}
