use arctis_audio::samples::{CHANNELS, Processor};
use arctis_core::eq::{Coefficients, PresetId, library};

fn designed(id: &str) -> Coefficients {
    let preset = library::find(&PresetId::new(id)).expect("built-in");
    Coefficients::design(preset.curve(), 48_000)
}

#[test]
fn running_and_taking_new_coefficients_allocate_nothing() {
    let (mut input_side, output) = triple_buffer::triple_buffer(&designed("flat"));
    let mut processor = Processor::new(output);
    let input = vec![0x3E; 1024 * CHANNELS * 4];
    let mut out = vec![0; input.len()];
    input_side.write(designed("v-shape"));
    let counted = allocation_counter::measure(|| {
        for _ in 0..8 {
            processor.run(&input, &mut out);
        }
    });
    assert_eq!(counted.count_total, 0);
}
