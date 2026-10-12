use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::values::OutOfRange;

#[test]
fn link_bytes_follow_hid_protocol_section_5_4() {
    let links: Vec<Result<Link, OutOfRange>> = (0..=4).map(Link::try_from).collect();
    assert_eq!(
        links,
        [
            Ok(Link::NotPaired),
            Ok(Link::Searching),
            Ok(Link::HeadsetOff),
            Ok(Link::Connected),
            Err(OutOfRange),
        ]
    );
}

#[test]
fn only_link_three_means_the_headset_is_connected() {
    let connected: Vec<Link> = Link::ALL
        .into_iter()
        .filter(|link| link.is_connected())
        .collect();
    assert_eq!(connected, [Link::Connected]);
}

#[test]
fn charging_bytes_follow_hid_protocol_section_5_4() {
    let states: Vec<Result<Charging, OutOfRange>> = (0..=4).map(Charging::try_from).collect();
    assert_eq!(
        states,
        [
            Ok(Charging::Unknown),
            Ok(Charging::Charging),
            Ok(Charging::PluggedIn),
            Ok(Charging::Discharging),
            Err(OutOfRange),
        ]
    );
}
