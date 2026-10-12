use arctis_core::battery::sample::{Sampler, UnixSeconds};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::time::Millis;

use crate::support::{discharging, reading, sample};

#[test]
fn unix_seconds_come_from_wall_clock_millis() {
    assert_eq!(
        UnixSeconds::from_millis(Millis(1_760_000_000_999)),
        UnixSeconds(1_760_000_000)
    );
}

#[test]
fn unix_seconds_subtract_without_wrapping() {
    assert_eq!(UnixSeconds(100).minus(30), UnixSeconds(70));
    assert_eq!(UnixSeconds(10).minus(30), UnixSeconds(0));
    assert_eq!(UnixSeconds(100).since(UnixSeconds(40)), 60);
    assert_eq!(UnixSeconds(40).since(UnixSeconds(100)), 0);
}

#[test]
fn only_a_connected_link_is_connected() {
    for link in Link::ALL {
        let connected = reading(50, Charging::Discharging, link).is_connected();
        assert_eq!(connected, link == Link::Connected, "{link:?}");
    }
}

#[test]
fn the_first_reading_is_recorded() {
    let mut sampler = Sampler::default();
    assert_eq!(
        sampler.observe(discharging(73), UnixSeconds(10)),
        Some(sample(10, discharging(73)))
    );
}

#[test]
fn an_unchanged_reading_is_not_recorded_again() {
    let mut sampler = Sampler::default();
    let _ = sampler.observe(discharging(73), UnixSeconds(10));
    assert_eq!(sampler.observe(discharging(73), UnixSeconds(20)), None);
}

#[test]
fn a_change_of_percent_charging_or_link_is_recorded() {
    let changes = [
        discharging(72),
        reading(73, Charging::Charging, Link::Connected),
        reading(73, Charging::Discharging, Link::HeadsetOff),
    ];
    for change in changes {
        let mut sampler = Sampler::default();
        let _ = sampler.observe(discharging(73), UnixSeconds(10));
        assert_eq!(
            sampler.observe(change, UnixSeconds(20)),
            Some(sample(20, change)),
            "{change:?}"
        );
    }
}

#[test]
fn history_seeds_the_last_reading() {
    let mut sampler = Sampler::starting_from(&[sample(1, discharging(73))]);
    assert_eq!(sampler.observe(discharging(73), UnixSeconds(30)), None);
}

#[test]
fn plugged_means_charging_or_plugged_in() {
    let cases = [
        (Charging::Charging, true),
        (Charging::PluggedIn, true),
        (Charging::Discharging, false),
        (Charging::Unknown, false),
    ];
    for (charging, plugged) in cases {
        assert_eq!(reading(50, charging, Link::Connected).is_plugged(), plugged);
    }
}
