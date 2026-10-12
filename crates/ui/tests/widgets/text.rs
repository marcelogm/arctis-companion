use arctis_core::battery::sample::UnixSeconds;
use arctis_core::eq::{Gain, Hertz, Q};
use arctis_core::testkit::snapshots::percent;
use arctis_core::view::text::{Arg, Text};
use arctis_ui::text::{NumberStyle, Resolver, optional, translated};

fn brazilian() -> Resolver {
    Resolver::new(NumberStyle::for_locale("pt_BR.UTF-8"))
}

fn american() -> Resolver {
    Resolver::new(NumberStyle::for_locale("en_US.UTF-8"))
}

fn decibels(half_db: i8) -> Text {
    let gain = Gain::from_half_db(half_db).expect("gain in the envelope");
    Text::format("{gain}", vec![("gain", Arg::Decibels(gain))])
}

#[test]
fn pt_br_renders_decibels_with_a_decimal_comma() {
    assert_eq!(brazilian().resolve(&decibels(4)), "+2,0 dB");
}

#[test]
fn q_renders_with_two_decimals_in_the_locale_style() {
    let q = Q::from_milli(1_414).expect("q in the envelope");
    let text = Text::format("Q {q}", vec![("q", Arg::Quality(q))]);
    assert_eq!(brazilian().resolve(&text), "Q 1,41");
    assert_eq!(american().resolve(&text), "Q 1.41");
}

#[test]
fn pt_br_renders_a_percent_with_a_space() {
    let text = Text::format("{level}", vec![("level", Arg::Percent(percent(73)))]);
    assert_eq!(brazilian().resolve(&text), "73 %");
}

#[test]
fn english_renders_decibels_with_a_decimal_point() {
    assert_eq!(american().resolve(&decibels(-3)), "-1.5 dB");
}

#[test]
fn the_c_locale_and_unknown_languages_use_a_point() {
    assert_eq!(NumberStyle::for_locale("C"), NumberStyle::POINT);
    assert_eq!(NumberStyle::for_locale("ja_JP.UTF-8"), NumberStyle::POINT);
}

#[test]
fn languages_with_a_decimal_comma_are_recognised_by_their_code() {
    assert_eq!(
        NumberStyle::for_locale("de_DE@euro"),
        NumberStyle::for_locale("pt_BR")
    );
    assert_ne!(NumberStyle::for_locale("pt"), NumberStyle::POINT);
}

#[test]
fn plurals_substitute_n() {
    let minutes = |n| Text::plural(["{n} minute", "{n} minutes"], n, Vec::new());
    assert_eq!(american().resolve(&minutes(1)), "1 minute");
    assert_eq!(american().resolve(&minutes(5)), "5 minutes");
}

#[test]
fn formats_fill_named_arguments() {
    let text = Text::format(
        "{preset}, chosen for {app}",
        vec![
            ("preset", Arg::Msgid("Flat".to_owned())),
            ("app", Arg::Str("Firefox".to_owned())),
        ],
    );
    assert_eq!(american().resolve(&text), "Flat, chosen for Firefox");
}

#[test]
fn argument_values_are_never_expanded_again() {
    let text = Text::format(
        "{preset}, chosen for {app}",
        vec![
            ("preset", Arg::Str("{app}".to_owned())),
            ("app", Arg::Str("Firefox".to_owned())),
        ],
    );
    assert_eq!(american().resolve(&text), "{app}, chosen for Firefox");
}

#[test]
fn unknown_and_unclosed_placeholders_stay_as_written() {
    let text = Text::format("{missing} and {open", vec![("app", Arg::Number(1))]);
    assert_eq!(american().resolve(&text), "{missing} and {open");
}

#[test]
fn verbatim_text_is_never_translated_or_filled() {
    assert_eq!(american().resolve(&Text::verbatim("{x} & y")), "{x} & y");
}

#[test]
fn numbers_and_frequencies_have_no_grouping() {
    let text = Text::format(
        "{count} at {frequency}",
        vec![
            ("count", Arg::Number(1000)),
            (
                "frequency",
                Arg::Hertz(Hertz::new(1000).expect("frequency")),
            ),
        ],
    );
    assert_eq!(brazilian().resolve(&text), "1000 at 1000 Hz");
}

#[test]
fn times_render_as_a_local_date() {
    let text = Text::format("{at}", vec![("at", Arg::Time(UnixSeconds(86_400)))]);
    assert!(!american().resolve(&text).is_empty());
}

#[test]
fn a_missing_optional_text_is_empty() {
    assert_eq!(optional(None), "");
    assert_eq!(optional(Some(&Text::plain("Not applied"))), "Not applied");
}

#[test]
fn a_fixed_label_without_a_catalogue_stays_english() {
    assert_eq!(translated("_Cancel"), "_Cancel");
}
