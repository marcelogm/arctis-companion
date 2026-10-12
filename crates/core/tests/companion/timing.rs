use arctis_core::companion::timing::{
    BATTERY_HYSTERESIS, CRITICAL_BATTERY, EDITOR_THROTTLE, QUIT_GRACE, RULE_ENTER, RULE_LEAVE,
    auto_switch_min_interval,
};
use arctis_core::messages::audio::SoundPath;

#[test]
fn constants_match_the_design() {
    assert_eq!(RULE_ENTER, 1_000);
    assert_eq!(RULE_LEAVE, 10_000);
    assert_eq!(EDITOR_THROTTLE, 250);
    assert_eq!(QUIT_GRACE, 1_000);
    assert_eq!(CRITICAL_BATTERY, 5);
    assert_eq!(BATTERY_HYSTERESIS, 3);
}

#[test]
fn each_path_has_its_switch_budget() {
    assert_eq!(auto_switch_min_interval(SoundPath::Dongle), 30_000);
    assert_eq!(auto_switch_min_interval(SoundPath::Bluetooth), 2_000);
}
