use arctis_core::eq::{Coefficients, Equalizer, PresetId, library};

fn designed(id: &str) -> Coefficients {
    let preset = library::find(&PresetId::new(id)).expect("built-in");
    Coefficients::design(preset.curve(), 48_000)
}

#[test]
fn process_and_retarget_never_allocate() {
    let flat = designed("flat");
    let shaped = designed("v-shape");
    let mut equalizer = Equalizer::new(&flat, 2);
    let mut block = vec![0.25_f32; 512];
    let counted = allocation_counter::measure(|| {
        equalizer.process(&mut block);
        equalizer.retarget(&shaped);
        equalizer.retarget(&flat);
        equalizer.retarget(&shaped);
        for _ in 0..8 {
            equalizer.process(&mut block);
        }
    });
    assert_eq!(counted.count_total, 0);
}
