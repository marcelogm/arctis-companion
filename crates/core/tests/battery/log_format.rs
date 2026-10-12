use arctis_core::battery::log_format::{
    RETENTION_SECONDS, parse_log, render_line, render_log, retain_recent,
};
use arctis_core::battery::sample::{Sample, UnixSeconds};
use arctis_core::protocol::status::{Charging, Link};

use crate::support::{discharging, reading, sample};

fn every_word() -> Vec<Sample> {
    Charging::ALL
        .into_iter()
        .flat_map(|charging| Link::ALL.into_iter().map(move |link| (charging, link)))
        .zip(1_u64..)
        .map(|((charging, link), at)| sample(at, reading(40, charging, link)))
        .collect()
}

#[test]
fn a_sample_renders_as_one_csv_line() {
    assert_eq!(
        render_line(&sample(1_760_000_000, discharging(73))),
        "1760000000,73,discharging,connected\n"
    );
}

#[test]
fn every_charging_and_link_word_round_trips() {
    let samples = every_word();
    assert_eq!(parse_log(&render_log(&samples)), samples);
}

#[test]
fn malformed_lines_are_skipped() {
    let lines = [
        "",
        "abc",
        "1,101,charging,connected",
        "1,50,flying,connected",
        "1,50,charging,orbiting",
        "1,50,charging,connected,extra",
        "1,50,charging",
        "-1,50,charging,connected",
    ];
    for line in lines {
        assert_eq!(parse_log(line), Vec::new(), "{line:?}");
    }
}

#[test]
fn good_lines_survive_their_bad_neighbours() {
    let text = "1,50,charging,connected\nnot a row\n2,51,charging,connected\n";
    assert_eq!(
        parse_log(text),
        vec![
            sample(1, reading(50, Charging::Charging, Link::Connected)),
            sample(2, reading(51, Charging::Charging, Link::Connected)),
        ]
    );
}

#[test]
fn rows_written_out_of_time_order_load_in_time_order() {
    let text = "20,50,discharging,connected\n10,51,discharging,connected\n";
    let times: Vec<UnixSeconds> = parse_log(text).iter().map(|row| row.at).collect();
    assert_eq!(times, [UnixSeconds(10), UnixSeconds(20)]);
}

#[test]
fn retention_keeps_exactly_eight_days() {
    let now = UnixSeconds(10 * RETENTION_SECONDS);
    let oldest_kept = now.minus(RETENTION_SECONDS).0;
    let samples = [
        sample(oldest_kept - 1, discharging(90)),
        sample(oldest_kept, discharging(80)),
        sample(now.0, discharging(70)),
    ];
    assert_eq!(retain_recent(&samples, now), samples[1..].to_vec());
}

#[test]
fn retention_is_eight_days() {
    assert_eq!(RETENTION_SECONDS, 8 * 24 * 60 * 60);
}
