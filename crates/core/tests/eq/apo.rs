use arctis_core::eq::apo::{self, ApoError, Clamp, Field, Import};
use arctis_core::eq::{Curve, FilterKind};

use crate::support::{filter, peaking};

const AUTOEQ: &str = "Preamp: -6.4 dB
Filter 1: ON LSC Fc 105 Hz Gain 5.5 dB Q 0.70
Filter 2: ON PK Fc 186 Hz Gain -3.2 dB Q 0.54
Filter 3: ON PK Fc 1310 Hz Gain 2.3 dB Q 1.67
Filter 4: ON PK Fc 3384 Hz Gain -4.1 dB Q 3.41
Filter 5: ON PK Fc 5928 Hz Gain 4.9 dB Q 4.57
Filter 6: ON HSC Fc 10000 Hz Gain -1.6 dB Q 0.70
";

#[test]
fn an_autoeq_file_lands_on_the_envelope_grid() {
    let expected = Curve::from_array([
        filter(FilterKind::LowShelf, 105, 11, 700),
        peaking(186, -6, 540),
        peaking(1_310, 5, 1_670),
        peaking(3_384, -8, 3_410),
        peaking(5_928, 10, 4_570),
        filter(FilterKind::HighShelf, 10_000, -3, 700),
    ]);
    let imported = apo::parse(AUTOEQ);
    assert_eq!(
        imported,
        Ok(Import {
            curve: expected,
            clamped: Vec::new()
        })
    );
}

#[test]
fn windows_line_endings_parse_alike() {
    assert_eq!(
        apo::parse(&AUTOEQ.replace('\n', "\r\n")),
        apo::parse(AUTOEQ)
    );
}

#[test]
fn values_outside_the_envelope_are_clamped_and_listed() {
    let text =
        "Filter 1: ON PK Fc 25000 Hz Gain 12 dB Q 15\nFilter 2: ON LPQ Fc 10 Hz Gain 3 dB Q 0.1\n";
    let import = apo::parse(text).expect("valid file");
    let expected = Curve::from_array([
        peaking(20_000, 20, 10_000),
        filter(FilterKind::LowPass, 20, 0, 200),
    ]);
    assert_eq!(import.curve, expected);
    let fields = [Field::Frequency, Field::Gain, Field::Q];
    let listed: Vec<Clamp> = [0, 1]
        .into_iter()
        .flat_map(|position| fields.map(|field| Clamp { position, field }))
        .collect();
    assert_eq!(import.clamped, listed);
}

#[test]
fn off_filters_import_disabled() {
    let import = apo::parse("Filter 1: OFF PK Fc 1000 Hz Gain 3 dB Q 1\n").expect("valid file");
    assert_eq!(
        import.curve,
        Curve::from_array([peaking(1_000, 6, 1_000).with_enabled(false)])
    );
}

#[test]
fn pass_filters_need_no_gain() {
    let import = apo::parse("Filter 1: ON HPQ Fc 80 Hz Q 0.71\n").expect("valid file");
    assert_eq!(
        import.curve,
        Curve::from_array([filter(FilterKind::HighPass, 80, 0, 710)])
    );
}

#[test]
fn other_lines_are_ignored() {
    let text =
        "# exported\nDevice: all\n\nPreamp: -3 dB\nFilter 1: ON PK Fc 1000 Hz Gain 3 dB Q 1\n";
    let curve = apo::parse(text).map(|import| import.curve);
    assert_eq!(curve, Ok(Curve::from_array([peaking(1_000, 6, 1_000)])));
}

#[test]
fn more_than_ten_filters_are_rejected() {
    let text = "Filter 1: ON PK Fc 1000 Hz Gain 1 dB Q 1\n".repeat(11);
    assert_eq!(apo::parse(&text), Err(ApoError::TooManyFilters(11)));
}

#[test]
fn a_file_without_filters_is_rejected() {
    assert_eq!(apo::parse("Preamp: -3 dB\n"), Err(ApoError::NoFilters));
}

#[test]
fn unknown_filter_types_are_rejected_with_their_line() {
    let text = "Preamp: 0 dB\nFilter 1: ON BP Fc 1000 Hz Q 1\n";
    assert_eq!(apo::parse(text), Err(ApoError::UnsupportedKind { line: 2 }));
}

#[test]
fn unreadable_filters_are_rejected_with_their_line() {
    let texts = [
        "Filter 1: ON PK Fc abc Hz Gain 1 dB Q 1",
        "Filter 1: ON PK Fc 100 Hz Q 1",
        "Filter 1: ON PK Fc 100 Hz Gain 1 dB",
        "Filter 1: MAYBE PK Fc 100 Hz Gain 1 dB Q 1",
        "Filter 1",
    ];
    for text in texts {
        assert_eq!(
            apo::parse(text),
            Err(ApoError::Malformed { line: 1 }),
            "{text}"
        );
    }
}
