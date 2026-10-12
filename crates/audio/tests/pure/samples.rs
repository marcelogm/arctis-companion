use arctis_audio::samples::{CHANNELS, FRAME_BYTES, Processor};
use arctis_core::eq::{Coefficients, Equalizer, PresetId, library};

fn designed(id: &str) -> Coefficients {
    let preset = library::find(&PresetId::new(id)).expect("built-in");
    Coefficients::design(preset.curve(), 48_000)
}

fn sine_bytes(frames: usize) -> Vec<u8> {
    (0..frames * CHANNELS)
        .map(|index| sine_sample(index / CHANNELS))
        .flat_map(f32::to_le_bytes)
        .collect()
}

fn sine_sample(frame: usize) -> f32 {
    let phase = f32::from(u16::try_from(frame % 48).expect("small")) / 48.0;
    (phase * std::f32::consts::TAU).sin() * 0.25
}

fn samples_of(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect()
}

#[test]
fn flat_passes_samples_through_bit_exact() {
    let (_, output) = triple_buffer::triple_buffer(&designed("flat"));
    let mut processor = Processor::new(output);
    let input = sine_bytes(256);
    let mut out = vec![0; input.len()];
    assert_eq!(processor.run(&input, &mut out), input.len());
    assert_eq!(out, input);
}

#[test]
fn only_whole_frames_are_processed() {
    let (_, output) = triple_buffer::triple_buffer(&designed("flat"));
    let mut processor = Processor::new(output);
    let input = sine_bytes(4);
    let mut out = vec![0xAA; input.len() + 3];
    assert_eq!(
        processor.run(&input[..FRAME_BYTES * 3 + 5], &mut out),
        FRAME_BYTES * 3
    );
    assert_eq!(out[FRAME_BYTES * 3..], [0xAA; FRAME_BYTES + 3]);
}

#[test]
fn published_coefficients_are_applied_like_a_retarget() {
    let (mut input_side, output) = triple_buffer::triple_buffer(&designed("flat"));
    let mut processor = Processor::new(output);
    let mut reference = Equalizer::new(&designed("flat"), CHANNELS);
    let first = sine_bytes(512);
    let second = sine_bytes(10_000);
    let mut ignored = vec![0; first.len()];
    processor.run(&first, &mut ignored);
    reference.process(&mut samples_of(&first));
    input_side.write(designed("v-shape"));
    reference.retarget(&designed("v-shape"));
    let mut out = vec![0; second.len()];
    processor.run(&second, &mut out);
    let mut expected = samples_of(&second);
    reference.process(&mut expected);
    assert_eq!(samples_of(&out), expected);
}
