use arctis_core::eq::library;
use arctis_core::eq::preset::{Preset, PresetId};
use arctis_core::l10n::{forms, msgid};
use arctis_core::testkit::english::english;
use arctis_core::view::text::{Arg, Text, preset_arg, preset_title};

use crate::support::{gain, hertz, percent};

fn custom(name: &str) -> Preset {
    Preset::custom(
        PresetId::new("mine"),
        name.to_owned(),
        library::flat().curve().clone(),
    )
}

#[test]
fn plain_text_is_its_msgid() {
    assert_eq!(english(&Text::plain("Sidetone")), "Sidetone");
}

#[test]
fn verbatim_text_is_kept_as_written() {
    assert_eq!(english(&Text::verbatim("Bass (Mine)")), "Bass (Mine)");
}

#[test]
fn format_binds_named_arguments() {
    let text = Text::format(
        "{preset}, chosen for {app}",
        vec![
            ("preset", Arg::Msgid("Footsteps".to_owned())),
            ("app", Arg::Str("Firefox".to_owned())),
        ],
    );
    assert_eq!(english(&text), "Footsteps, chosen for Firefox");
}

#[test]
fn plural_picks_the_form_and_binds_n() {
    let forms = ["{n} minute", "{n} minutes"];
    assert_eq!(english(&Text::plural(forms, 1, Vec::new())), "1 minute");
    assert_eq!(english(&Text::plural(forms, 5, Vec::new())), "5 minutes");
}

#[test]
fn arguments_render_with_their_units() {
    let cases = [
        (Arg::Percent(percent(73)), "73 %"),
        (Arg::Decibels(gain(4)), "+2.0 dB"),
        (Arg::Decibels(gain(-3)), "-1.5 dB"),
        (Arg::Hertz(hertz(250)), "250 Hz"),
        (Arg::Number(7), "7"),
    ];
    for (arg, expected) in cases {
        assert_eq!(english(&Text::format("{x}", vec![("x", arg)])), expected);
    }
}

#[test]
fn built_in_names_are_msgids_and_custom_names_are_user_text() {
    assert_eq!(preset_title(&library::flat()), Text::plain("Flat"));
    assert_eq!(preset_title(&custom("Mine")), Text::verbatim("Mine"));
    assert_eq!(preset_arg(&library::flat()), Arg::Msgid("Flat".to_owned()));
    assert_eq!(preset_arg(&custom("Mine")), Arg::Str("Mine".to_owned()));
}

#[test]
#[should_panic(expected = "unbound placeholder")]
fn english_rejects_an_unbound_placeholder() {
    let _ = english(&Text::format("{missing} remaining", Vec::new()));
}

#[test]
fn user_text_may_contain_braces() {
    let text = Text::format(
        "{preset}, chosen by hand",
        vec![("preset", Arg::Str("{Mine}".to_owned()))],
    );
    assert_eq!(english(&text), "{Mine}, chosen by hand");
}

#[test]
fn markers_return_their_english_text() {
    assert_eq!(msgid("Low"), "Low");
    assert_eq!(
        forms("{n} minute", "{n} minutes"),
        ["{n} minute", "{n} minutes"]
    );
}

#[test]
fn markers_work_in_constants() {
    const MARKS: [(u8, &str); 1] = [(0, msgid("Off"))];
    assert_eq!(MARKS[0].1, "Off");
}
