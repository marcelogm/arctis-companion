use super::biquad::SectionState;
use super::coefficients::{ChannelState, Coefficients};
use super::curve::MAX_FILTERS;

const FADES_PER_SECOND: u32 = 100;

pub struct Equalizer {
    current: Coefficients,
    target: Coefficients,
    pending: Option<Coefficients>,
    current_states: Vec<ChannelState>,
    target_states: Vec<ChannelState>,
    fade: Option<Fade>,
}

impl Equalizer {
    #[must_use]
    pub fn new(coefficients: &Coefficients, channels: usize) -> Self {
        let silence = vec![[SectionState::default(); MAX_FILTERS]; channels];
        Self {
            current: *coefficients,
            target: *coefficients,
            pending: None,
            current_states: silence.clone(),
            target_states: silence,
            fade: None,
        }
    }

    pub fn retarget(&mut self, coefficients: &Coefficients) {
        if self.fade.is_some() {
            self.pending = Some(*coefficients);
        } else {
            self.start_fade(*coefficients);
        }
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        let channels = self.current_states.len();
        if channels == 0 {
            return;
        }
        for frame in samples.chunks_mut(channels) {
            self.process_frame(frame);
        }
    }

    fn process_frame(&mut self, frame: &mut [f32]) {
        match self.fade {
            None => self.steady(frame),
            Some(fade) => self.crossfade(frame, fade),
        }
    }

    fn steady(&mut self, frame: &mut [f32]) {
        for (sample, state) in frame.iter_mut().zip(&mut self.current_states) {
            *sample = self.current.tick(state, *sample);
        }
    }

    fn crossfade(&mut self, frame: &mut [f32], fade: Fade) {
        let weight = fade.weight();
        let states = self
            .current_states
            .iter_mut()
            .zip(self.target_states.iter_mut());
        for (sample, (from, to)) in frame.iter_mut().zip(states) {
            let old = self.current.tick(from, *sample);
            let new = self.target.tick(to, *sample);
            *sample = old + (new - old) * weight;
        }
        self.advance(fade);
    }

    fn advance(&mut self, fade: Fade) {
        let next = fade.advanced();
        if next.is_complete() {
            self.finish_fade();
        } else {
            self.fade = Some(next);
        }
    }

    fn start_fade(&mut self, coefficients: Coefficients) {
        self.target = coefficients;
        self.target_states.copy_from_slice(&self.current_states);
        self.fade = Some(Fade::over(fade_length(coefficients.rate())));
    }

    fn finish_fade(&mut self) {
        self.current = self.target;
        std::mem::swap(&mut self.current_states, &mut self.target_states);
        self.silence_unused_sections();
        self.fade = None;
        if let Some(pending) = self.pending.take() {
            self.start_fade(pending);
        }
    }

    fn silence_unused_sections(&mut self) {
        let used = self.current.sections().len();
        for state in &mut self.current_states {
            state[used..].fill(SectionState::default());
        }
    }
}

#[derive(Clone, Copy)]
struct Fade {
    elapsed: u16,
    length: u16,
}

impl Fade {
    fn over(length: u16) -> Self {
        Self {
            elapsed: 0,
            length: length.max(1),
        }
    }

    fn weight(self) -> f32 {
        f32::from(self.elapsed + 1) / f32::from(self.length)
    }

    fn advanced(self) -> Self {
        Self {
            elapsed: self.elapsed + 1,
            ..self
        }
    }

    fn is_complete(self) -> bool {
        self.elapsed >= self.length
    }
}

fn fade_length(rate: u32) -> u16 {
    u16::try_from(rate / FADES_PER_SECOND).unwrap_or(u16::MAX)
}
