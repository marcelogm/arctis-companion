use super::biquad::SectionState;
use super::coefficients::{ChannelState, Coefficients};
use super::curve::MAX_FILTERS;

pub struct Equalizer {
    current: Coefficients,
    current_states: Vec<ChannelState>,
}

impl Equalizer {
    #[must_use]
    pub fn new(coefficients: &Coefficients, channels: usize) -> Self {
        Self {
            current: *coefficients,
            current_states: vec![[SectionState::default(); MAX_FILTERS]; channels],
        }
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        let channels = self.current_states.len();
        if channels == 0 {
            return;
        }
        for frame in samples.chunks_mut(channels) {
            self.steady(frame);
        }
    }

    fn steady(&mut self, frame: &mut [f32]) {
        for (sample, state) in frame.iter_mut().zip(&mut self.current_states) {
            *sample = self.current.tick(state, *sample);
        }
    }
}
