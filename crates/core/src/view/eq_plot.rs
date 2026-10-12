use super::eq_graph::{HIGH_HZ, Handle, LOW_HZ, LevelRange};
use super::preset_edits::BandEdit;
use super::text::Text;
use crate::eq::filter::Filter;
use crate::l10n::msgid;

pub const HANDLE_RADIUS: f32 = 12.0;
pub const GUTTER_LEFT: f32 = 36.0;
pub const GUTTER_RIGHT: f32 = 12.0;
pub const GUTTER_TOP: f32 = 8.0;
pub const GUTTER_BOTTOM: f32 = 26.0;
pub const ZONE_LABEL_GAP: f32 = 6.0;
const MIN_LEVEL_SPACING: f32 = 24.0;
const LEVEL_TOLERANCE: f32 = 1.0e-3;
const LEVEL_STEPS: [f32; 5] = [3.0, 6.0, 12.0, 24.0, 48.0];
const DECADES: [f32; 4] = [10.0, 100.0, 1_000.0, 10_000.0];
const MAJOR_MULTIPLES: [u8; 3] = [1, 2, 5];
const SEMITONE: f64 = 1.059_463_094_359_295_3;
const Q_STEP: f64 = 1.189_207_115_002_721;
const GAIN_STEP_DB: f64 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plot {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub levels: LevelRange,
}

impl Plot {
    #[must_use]
    pub fn within(width: f32, height: f32, levels: LevelRange) -> Self {
        Self {
            left: GUTTER_LEFT,
            top: GUTTER_TOP,
            width: (width - GUTTER_LEFT - GUTTER_RIGHT).max(0.0),
            height: (height - GUTTER_TOP - GUTTER_BOTTOM).max(0.0),
            levels,
        }
    }

    #[must_use]
    pub fn right(&self) -> f32 {
        self.left + self.width
    }

    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.top + self.height
    }

    #[must_use]
    pub fn x_of(&self, hz: f32) -> f32 {
        let fraction = (hz / LOW_HZ).log10() / (HIGH_HZ / LOW_HZ).log10();
        self.left + self.width * unit(fraction)
    }

    #[must_use]
    pub fn hz_at(&self, x: f32) -> f32 {
        let fraction = unit((x - self.left) / self.width.max(f32::EPSILON));
        LOW_HZ * (HIGH_HZ / LOW_HZ).powf(fraction)
    }

    #[must_use]
    pub fn y_of(&self, db: f32) -> f32 {
        let fraction = (self.levels.high_db - db) / self.levels.span();
        self.top + self.height * unit(fraction)
    }

    #[must_use]
    pub fn db_at(&self, y: f32) -> f32 {
        let fraction = unit((y - self.top) / self.height.max(f32::EPSILON));
        self.levels.high_db - self.levels.span() * fraction
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emphasis {
    Minor,
    Major,
    Zero,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tick {
    pub value: f32,
    pub label: Option<Text>,
    pub emphasis: Emphasis,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Zone {
    pub name: Text,
    pub low_hz: f32,
    pub high_hz: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nudge {
    Lower,
    Higher,
    Up,
    Down,
    Narrower,
    Wider,
}

#[must_use]
pub fn frequency_ticks() -> Vec<Tick> {
    DECADES
        .into_iter()
        .flat_map(|decade| (1_u8..=9).map(move |multiple| (decade, multiple)))
        .map(|(decade, multiple)| frequency_tick(decade * f32::from(multiple), multiple))
        .filter(|tick| (LOW_HZ..=HIGH_HZ).contains(&tick.value))
        .collect()
}

#[must_use]
pub fn level_ticks(levels: LevelRange, height: f32) -> Vec<Tick> {
    let step = level_step(levels, height);
    let first = (levels.low_db / step).ceil() * step;
    std::iter::successors(Some(first), |value| Some(value + step))
        .take_while(|value| *value <= levels.high_db + LEVEL_TOLERANCE)
        .map(level_tick)
        .collect()
}

#[must_use]
pub fn zones() -> Vec<Zone> {
    [
        (msgid("Sub-bass"), 20.0, 60.0),
        (msgid("Bass"), 60.0, 250.0),
        (msgid("Low mids"), 250.0, 500.0),
        (msgid("Mids"), 500.0, 2_000.0),
        (msgid("High mids"), 2_000.0, 4_000.0),
        (msgid("Presence"), 4_000.0, 6_000.0),
        (msgid("Brilliance"), 6_000.0, 20_000.0),
    ]
    .into_iter()
    .map(|(name, low_hz, high_hz)| Zone {
        name: Text::plain(name),
        low_hz,
        high_hz,
    })
    .collect()
}

#[must_use]
pub fn zone_label_centre(plot: &Plot, zone: &Zone, label_width: f32) -> f32 {
    let centre = plot.x_of((zone.low_hz * zone.high_hz).sqrt());
    let half = label_width / 2.0;
    centre.clamp(
        plot.left + half,
        (plot.right() - half).max(plot.left + half),
    )
}

#[must_use]
pub fn zone_label_rows(plot: &Plot, label_widths: &[f32]) -> Option<usize> {
    let zones = zones();
    if label_widths.len() != zones.len() {
        return None;
    }
    let spans: Vec<(f32, f32)> = zones
        .iter()
        .zip(label_widths)
        .map(|(zone, width)| {
            let centre = zone_label_centre(plot, zone, *width);
            (centre - width / 2.0, centre + width / 2.0)
        })
        .collect();
    [1, 2].into_iter().find(|rows| rows_fit(&spans, *rows))
}

#[must_use]
pub fn hit_handle(plot: &Plot, handles: &[Handle], x: f32, y: f32) -> Option<usize> {
    handles
        .iter()
        .map(|handle| (handle.index, distance(plot, handle, x, y)))
        .filter(|(_, distance)| *distance <= HANDLE_RADIUS)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(index, _)| index)
}

#[must_use]
pub fn drag_edit(plot: &Plot, start: (f32, f32), offset: (f32, f32)) -> BandEdit {
    BandEdit::Point {
        hz: f64::from(plot.hz_at(start.0 + offset.0)),
        db: f64::from(plot.db_at(start.1 + offset.1)),
    }
}

#[must_use]
pub fn nudged(filter: Filter, nudge: Nudge) -> Option<BandEdit> {
    let hz = f64::from(filter.hz().get());
    let gain = filter.kind().has_gain().then(|| filter.gain().db());
    match nudge {
        Nudge::Higher => Some(BandEdit::Frequency(hz * SEMITONE)),
        Nudge::Lower => Some(BandEdit::Frequency(hz / SEMITONE)),
        Nudge::Up => gain.map(|db| BandEdit::Gain(db + GAIN_STEP_DB)),
        Nudge::Down => gain.map(|db| BandEdit::Gain(db - GAIN_STEP_DB)),
        Nudge::Narrower => Some(scrolled_q(filter, 1.0)),
        Nudge::Wider => Some(scrolled_q(filter, -1.0)),
    }
}

#[must_use]
pub fn scrolled_q(filter: Filter, steps: f64) -> BandEdit {
    BandEdit::Q(filter.q().value() * Q_STEP.powf(steps))
}

fn rows_fit(spans: &[(f32, f32)], rows: usize) -> bool {
    spans
        .iter()
        .zip(spans.iter().skip(rows))
        .all(|(before, after)| before.1 + ZONE_LABEL_GAP <= after.0)
}

fn distance(plot: &Plot, handle: &Handle, x: f32, y: f32) -> f32 {
    (plot.x_of(handle.hz) - x).hypot(plot.y_of(handle.db) - y)
}

fn level_step(levels: LevelRange, height: f32) -> f32 {
    let spacing = |step: f32| height * step / levels.span();
    LEVEL_STEPS
        .into_iter()
        .find(|step| spacing(*step) >= MIN_LEVEL_SPACING)
        .unwrap_or(LEVEL_STEPS[LEVEL_STEPS.len() - 1])
}

fn level_tick(value: f32) -> Tick {
    let zero = value.abs() < LEVEL_TOLERANCE;
    let label = match value {
        _ if zero => "0".to_owned(),
        v if v > 0.0 => format!("+{v}"),
        v => format!("−{}", -v),
    };
    Tick {
        value: if zero { 0.0 } else { value },
        label: Some(Text::verbatim(&label)),
        emphasis: if zero {
            Emphasis::Zero
        } else {
            Emphasis::Major
        },
    }
}

fn frequency_tick(value: f32, multiple: u8) -> Tick {
    let major = MAJOR_MULTIPLES.contains(&multiple);
    Tick {
        value,
        label: major.then(|| Text::verbatim(&frequency_label(value))),
        emphasis: if major {
            Emphasis::Major
        } else {
            Emphasis::Minor
        },
    }
}

fn frequency_label(hz: f32) -> String {
    if hz >= 1_000.0 {
        format!("{}k", hz / 1_000.0)
    } else {
        format!("{hz}")
    }
}

fn unit(fraction: f32) -> f32 {
    if fraction.is_nan() {
        0.0
    } else {
        fraction.clamp(0.0, 1.0)
    }
}
