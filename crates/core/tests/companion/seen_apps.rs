use arctis_core::companion::seen_apps::{MAX_SEEN_APPS, SeenApp, newly_playing, remember};
use arctis_core::messages::audio::{AppIdentifiers, SoundPath, StreamInfo};

use crate::support::stream;

fn app(key: &str) -> SeenApp {
    SeenApp {
        key: key.to_owned(),
        name: key.to_owned(),
        identifiers: vec![key.to_owned()],
    }
}

fn keys(apps: &[SeenApp]) -> Vec<&str> {
    apps.iter().map(|app| app.key.as_str()).collect()
}

#[test]
fn remembering_puts_the_newest_app_first_once() {
    let apps = remember(&[app("a"), app("b")], app("b"));
    assert_eq!(keys(&apps), ["b", "a"]);
}

#[test]
fn at_most_fifty_apps_are_remembered() {
    let many: Vec<SeenApp> = (0..MAX_SEEN_APPS)
        .map(|n| app(&format!("app{n}")))
        .collect();
    let apps = remember(&many, app("new"));
    assert_eq!(apps.len(), MAX_SEEN_APPS);
    assert_eq!(apps.first().map(|a| a.key.as_str()), Some("new"));
    assert_eq!(MAX_SEEN_APPS, 50);
}

#[test]
fn an_app_that_starts_playing_is_seen() {
    let seen = newly_playing(&[], &[stream("firefox", Some(SoundPath::Dongle), true)]);
    assert_eq!(
        seen,
        [SeenApp {
            key: "org.firefox.App".to_owned(),
            name: "firefox".to_owned(),
            identifiers: vec!["org.firefox.App".to_owned(), "firefox".to_owned()],
        }]
    );
}

#[test]
fn an_app_already_playing_is_not_seen_again() {
    let playing = [stream("firefox", Some(SoundPath::Dongle), true)];
    assert!(newly_playing(&playing, &playing).is_empty());
}

#[test]
fn a_paused_stream_is_not_playing() {
    assert!(newly_playing(&[], &[stream("firefox", None, false)]).is_empty());
}

#[test]
fn an_app_playing_on_another_output_is_seen() {
    assert_eq!(newly_playing(&[], &[stream("mpv", None, true)]).len(), 1);
}

#[test]
fn two_streams_of_one_app_are_seen_once() {
    let streams = [
        stream("firefox", Some(SoundPath::Dongle), true),
        stream("firefox", Some(SoundPath::Dongle), true),
    ];
    assert_eq!(newly_playing(&[], &streams).len(), 1);
}

#[test]
fn a_stream_without_identifiers_is_ignored() {
    let anonymous = StreamInfo {
        ids: AppIdentifiers::new(None, None, None),
        name: String::new(),
        running: true,
        path: None,
    };
    assert!(newly_playing(&[], &[anonymous]).is_empty());
}

#[test]
fn an_app_answers_to_any_identifier_in_any_case() {
    let firefox = SeenApp {
        key: "org.mozilla.firefox".to_owned(),
        name: "Firefox".to_owned(),
        identifiers: vec!["org.mozilla.firefox".to_owned(), "firefox".to_owned()],
    };
    assert!(firefox.answers_to("FIREFOX"));
    assert!(!firefox.answers_to("chromium"));
}
