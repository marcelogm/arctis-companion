use std::f64::consts::TAU;

use super::filter::{Filter, FilterKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub kind: FilterKind,
    pub hz: f64,
    pub gain_db: f64,
    pub q: f64,
}

impl From<Filter> for Params {
    fn from(filter: Filter) -> Self {
        Self {
            kind: filter.kind(),
            hz: f64::from(filter.hz().get()),
            gain_db: filter.gain().db(),
            q: filter.q().value(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Biquad {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
}

impl Biquad {
    #[must_use]
    pub fn design(params: &Params, rate: f64) -> Self {
        let shape = Shape::new(params, rate);
        let [b0, b1, b2, a0, a1, a2] = match params.kind {
            FilterKind::Peaking => shape.peaking(),
            FilterKind::LowShelf => shape.low_shelf(),
            FilterKind::HighShelf => shape.high_shelf(),
            FilterKind::LowPass => shape.low_pass(),
            FilterKind::HighPass => shape.high_pass(),
        };
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    #[must_use]
    pub fn magnitude_db(&self, hz: f64, rate: f64) -> f64 {
        let omega = TAU * hz / rate;
        let numerator = power_at([self.b0, self.b1, self.b2], omega);
        let denominator = power_at([1.0, self.a1, self.a2], omega);
        10.0 * (numerator / denominator).log10()
    }
}

fn power_at([c0, c1, c2]: [f64; 3], omega: f64) -> f64 {
    let real = c0 + c1 * omega.cos() + c2 * (2.0 * omega).cos();
    let imaginary = c1 * omega.sin() + c2 * (2.0 * omega).sin();
    real * real + imaginary * imaginary
}

#[derive(Clone, Copy)]
struct Shape {
    amplitude: f64,
    cos: f64,
    alpha: f64,
}

impl Shape {
    fn new(params: &Params, rate: f64) -> Self {
        let omega = TAU * params.hz / rate;
        Self {
            amplitude: 10f64.powf(params.gain_db / 40.0),
            cos: omega.cos(),
            alpha: omega.sin() / (2.0 * params.q),
        }
    }

    fn peaking(self) -> [f64; 6] {
        let Self {
            amplitude: a,
            cos,
            alpha,
        } = self;
        [
            1.0 + alpha * a,
            -2.0 * cos,
            1.0 - alpha * a,
            1.0 + alpha / a,
            -2.0 * cos,
            1.0 - alpha / a,
        ]
    }

    fn low_shelf(self) -> [f64; 6] {
        let Self {
            amplitude: a,
            cos,
            alpha,
        } = self;
        let root = 2.0 * a.sqrt() * alpha;
        [
            a * ((a + 1.0) - (a - 1.0) * cos + root),
            2.0 * a * ((a - 1.0) - (a + 1.0) * cos),
            a * ((a + 1.0) - (a - 1.0) * cos - root),
            (a + 1.0) + (a - 1.0) * cos + root,
            -2.0 * ((a - 1.0) + (a + 1.0) * cos),
            (a + 1.0) + (a - 1.0) * cos - root,
        ]
    }

    fn high_shelf(self) -> [f64; 6] {
        let Self {
            amplitude: a,
            cos,
            alpha,
        } = self;
        let root = 2.0 * a.sqrt() * alpha;
        [
            a * ((a + 1.0) + (a - 1.0) * cos + root),
            -2.0 * a * ((a - 1.0) + (a + 1.0) * cos),
            a * ((a + 1.0) + (a - 1.0) * cos - root),
            (a + 1.0) - (a - 1.0) * cos + root,
            2.0 * ((a - 1.0) - (a + 1.0) * cos),
            (a + 1.0) - (a - 1.0) * cos - root,
        ]
    }

    fn low_pass(self) -> [f64; 6] {
        let Self { cos, alpha, .. } = self;
        let edge = f64::midpoint(1.0, -cos);
        [edge, 1.0 - cos, edge, 1.0 + alpha, -2.0 * cos, 1.0 - alpha]
    }

    fn high_pass(self) -> [f64; 6] {
        let Self { cos, alpha, .. } = self;
        let edge = f64::midpoint(1.0, cos);
        [
            edge,
            -(1.0 + cos),
            edge,
            1.0 + alpha,
            -2.0 * cos,
            1.0 - alpha,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Section {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SectionState {
    s1: f32,
    s2: f32,
}

impl Section {
    pub const IDENTITY: Self = Self {
        b0: 1.0,
        b1: 0.0,
        b2: 0.0,
        a1: 0.0,
        a2: 0.0,
    };

    pub fn tick(self, state: &mut SectionState, input: f32) -> f32 {
        let output = self.b0 * input + state.s1;
        state.s1 = self.b1 * input - self.a1 * output + state.s2;
        state.s2 = self.b2 * input - self.a2 * output;
        output
    }

    #[must_use]
    pub fn poles_inside_unit_circle(self) -> bool {
        let a1 = f64::from(self.a1);
        let a2 = f64::from(self.a2);
        a2.abs() < 1.0 && a1.abs() < 1.0 + a2
    }
}

impl From<Biquad> for Section {
    fn from(biquad: Biquad) -> Self {
        Self {
            b0: narrow(biquad.b0),
            b1: narrow(biquad.b1),
            b2: narrow(biquad.b2),
            a1: narrow(biquad.a1),
            a2: narrow(biquad.a2),
        }
    }
}

#[expect(clippy::cast_possible_truncation)]
pub(crate) fn narrow(value: f64) -> f32 {
    value as f32
}
