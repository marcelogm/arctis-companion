use arctis_core::eq::{Coefficients, Curve};
use pipewire::spa::param::audio::AudioInfoRaw;
use pipewire::spa::pod::Pod;
use triple_buffer::{Input, Output};

pub const INITIAL_RATE: u32 = 48_000;

pub struct Designer {
    curve: Curve,
    rate: u32,
    coefficients: Input<Coefficients>,
}

impl Designer {
    #[must_use]
    pub fn new(curve: &Curve) -> (Self, Output<Coefficients>) {
        let (coefficients, output) =
            triple_buffer::triple_buffer(&Coefficients::design(curve, INITIAL_RATE));
        let designer = Self {
            curve: curve.clone(),
            rate: INITIAL_RATE,
            coefficients,
        };
        (designer, output)
    }

    pub fn set_curve(&mut self, curve: &Curve) {
        self.curve = curve.clone();
        self.publish();
    }

    pub fn set_rate(&mut self, rate: u32) {
        if rate != self.rate {
            self.rate = rate;
            self.publish();
        }
    }

    fn publish(&mut self) {
        self.coefficients
            .write(Coefficients::design(&self.curve, self.rate));
    }
}

#[must_use]
pub fn rate_of(format: &Pod) -> Option<u32> {
    let mut info = AudioInfoRaw::new();
    info.parse(format).ok()?;
    Some(info.rate()).filter(|rate| *rate > 0)
}
