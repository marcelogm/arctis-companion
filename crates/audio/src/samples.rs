use arctis_core::eq::{Coefficients, Equalizer};
use triple_buffer::Output;

pub const CHANNELS: usize = 2;
pub const SAMPLE_BYTES: usize = 4;
pub const FRAME_BYTES: usize = CHANNELS * SAMPLE_BYTES;
const BLOCK_SAMPLES: usize = 4096 * CHANNELS;

pub struct Processor {
    equalizer: Equalizer,
    coefficients: Output<Coefficients>,
    block: Vec<f32>,
}

impl Processor {
    #[must_use]
    pub fn new(coefficients: Output<Coefficients>) -> Self {
        Self {
            equalizer: Equalizer::new(coefficients.output_buffer(), CHANNELS),
            coefficients,
            block: vec![0.0; BLOCK_SAMPLES],
        }
    }

    pub fn run(&mut self, input: &[u8], output: &mut [u8]) -> usize {
        if self.coefficients.update() {
            self.equalizer.retarget(self.coefficients.output_buffer());
        }
        let length = whole_frames(input.len().min(output.len()));
        let block_bytes = BLOCK_SAMPLES * SAMPLE_BYTES;
        let blocks = input[..length]
            .chunks(block_bytes)
            .zip(output[..length].chunks_mut(block_bytes));
        for (from, to) in blocks {
            self.run_block(from, to);
        }
        length
    }

    fn run_block(&mut self, input: &[u8], output: &mut [u8]) {
        let samples = &mut self.block[..input.len() / SAMPLE_BYTES];
        for (sample, bytes) in samples.iter_mut().zip(input.as_chunks::<SAMPLE_BYTES>().0) {
            *sample = f32::from_le_bytes(*bytes);
        }
        self.equalizer.process(samples);
        for (bytes, sample) in output
            .as_chunks_mut::<SAMPLE_BYTES>()
            .0
            .iter_mut()
            .zip(samples)
        {
            *bytes = sample.to_le_bytes();
        }
    }
}

fn whole_frames(bytes: usize) -> usize {
    bytes - bytes % FRAME_BYTES
}
