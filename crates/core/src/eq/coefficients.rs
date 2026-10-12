use super::biquad::{Biquad, Params, Section, SectionState, narrow};
use super::curve::{Curve, MAX_FILTERS};
use super::filter::Filter;
use super::response::preamp_db;

pub type ChannelState = [SectionState; MAX_FILTERS];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coefficients {
    sections: [Section; MAX_FILTERS],
    count: usize,
    preamp: f32,
    rate: u32,
}

impl Coefficients {
    #[must_use]
    pub fn design(curve: &Curve, rate: u32) -> Self {
        let designed = curve
            .filters()
            .iter()
            .copied()
            .filter(|filter| filter.shapes_sound() && below_nyquist(*filter, rate))
            .map(|filter| Section::from(Biquad::design(&Params::from(filter), f64::from(rate))));
        let mut sections = [Section::IDENTITY; MAX_FILTERS];
        let mut count = 0;
        for (slot, section) in sections.iter_mut().zip(designed) {
            *slot = section;
            count += 1;
        }
        let preamp = narrow(10f64.powf(preamp_db(curve) / 20.0));
        Self {
            sections,
            count,
            preamp,
            rate,
        }
    }

    #[must_use]
    pub fn sections(&self) -> &[Section] {
        &self.sections[..self.count]
    }

    #[must_use]
    pub fn preamp(&self) -> f32 {
        self.preamp
    }

    #[must_use]
    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn tick(&self, state: &mut ChannelState, input: f32) -> f32 {
        let shaped = self
            .sections()
            .iter()
            .zip(state.iter_mut())
            .fold(input, |sample, (section, section_state)| {
                section.tick(section_state, sample)
            });
        shaped * self.preamp
    }
}

fn below_nyquist(filter: Filter, rate: u32) -> bool {
    2 * u32::from(filter.hz().get()) < rate
}
