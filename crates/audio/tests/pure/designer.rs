use arctis_audio::designer::{Designer, INITIAL_RATE, rate_of};
use arctis_audio::pod::format_pod;
use arctis_core::eq::{Coefficients, Curve, PresetId, library};
use pipewire::spa::pod::Pod;

fn curve(id: &str) -> Curve {
    library::find(&PresetId::new(id))
        .expect("built-in")
        .curve()
        .clone()
}

#[test]
fn a_negotiated_format_names_its_rate() {
    for (rate, expected) in [(44_100, Some(44_100)), (48_000, Some(48_000)), (0, None)] {
        let bytes = format_pod(Some(rate));
        let pod = Pod::from_bytes(&bytes).expect("pod");
        assert_eq!(rate_of(pod), expected, "{rate}");
    }
}

#[test]
fn coefficients_start_at_the_initial_rate() {
    let (_designer, output) = Designer::new(&curve("v-shape"));
    assert_eq!(
        *output.output_buffer(),
        Coefficients::design(&curve("v-shape"), INITIAL_RATE)
    );
}

#[test]
fn a_new_rate_publishes_coefficients_for_it() {
    let (mut designer, mut output) = Designer::new(&curve("v-shape"));
    designer.set_rate(44_100);
    assert!(output.update());
    assert_eq!(
        *output.output_buffer(),
        Coefficients::design(&curve("v-shape"), 44_100)
    );
}

#[test]
fn the_same_rate_publishes_nothing() {
    let (mut designer, mut output) = Designer::new(&curve("v-shape"));
    designer.set_rate(INITIAL_RATE);
    assert!(!output.update());
}

#[test]
fn a_new_curve_is_designed_at_the_negotiated_rate() {
    let (mut designer, mut output) = Designer::new(&curve("flat"));
    designer.set_rate(44_100);
    designer.set_curve(&curve("v-shape"));
    assert!(output.update());
    assert_eq!(
        *output.output_buffer(),
        Coefficients::design(&curve("v-shape"), 44_100)
    );
}
