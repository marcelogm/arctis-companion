use arctis_core::settings::values::{
    AutoPowerOff, MicVolume, MuteLight, OutOfRange, Percent, Sidetone, WirelessMode,
};

#[test]
fn mic_volume_spans_zero_to_fifteen() {
    assert_eq!(MicVolume::try_from(0).map(MicVolume::get), Ok(0));
    assert_eq!(MicVolume::try_from(15).map(MicVolume::get), Ok(15));
    assert_eq!(MicVolume::try_from(16), Err(OutOfRange));
}

#[test]
fn sidetone_spans_zero_to_ten() {
    assert_eq!(Sidetone::try_from(10).map(Sidetone::get), Ok(10));
    assert_eq!(Sidetone::try_from(11), Err(OutOfRange));
}

#[test]
fn percent_spans_zero_to_one_hundred() {
    assert_eq!(Percent::try_from(100).map(Percent::get), Ok(100));
    assert_eq!(Percent::try_from(101), Err(OutOfRange));
}

#[test]
fn mute_light_levels_use_the_raw_bytes_0_1_4_10() {
    let raws: Vec<u8> = MuteLight::ALL.into_iter().map(MuteLight::raw).collect();
    assert_eq!(raws, [0, 1, 4, 10]);
}

#[test]
fn mute_light_reads_every_level_back_from_its_raw_byte() {
    for level in MuteLight::ALL {
        assert_eq!(MuteLight::from_raw(level.raw()), level);
    }
}

#[test]
fn unknown_mute_light_bytes_read_as_medium_like_the_firmware() {
    assert_eq!(MuteLight::from_raw(7), MuteLight::Medium);
}

#[test]
fn wireless_mode_zero_is_extended_range() {
    assert_eq!(WirelessMode::try_from(0), Ok(WirelessMode::ExtendedRange));
    assert_eq!(WirelessMode::try_from(1), Ok(WirelessMode::Standard));
    assert_eq!(WirelessMode::try_from(2), Err(OutOfRange));
}

#[test]
fn wireless_mode_raw_bytes_round_trip() {
    for mode in WirelessMode::ALL {
        assert_eq!(WirelessMode::try_from(mode.raw()), Ok(mode));
    }
}

#[test]
fn auto_power_off_accepts_only_the_ten_choices() {
    let accepted: Vec<u8> = (0..=u8::MAX)
        .filter(|minutes| AutoPowerOff::try_from(*minutes).is_ok())
        .collect();
    assert_eq!(accepted, [0, 1, 5, 10, 15, 30, 45, 60, 75, 90]);
}

#[test]
fn auto_power_off_never_is_zero_minutes() {
    assert_eq!(AutoPowerOff::NEVER.minutes(), 0);
    assert_eq!(
        AutoPowerOff::try_from(30).map(AutoPowerOff::minutes),
        Ok(30)
    );
}

#[test]
fn unlisted_auto_power_off_minutes_read_as_the_nearest_choice() {
    let nearest: Vec<u8> = [0, 2, 3, 20, 200]
        .into_iter()
        .map(|minutes| AutoPowerOff::nearest(minutes).minutes())
        .collect();
    assert_eq!(nearest, [0, 1, 1, 15, 90]);
}
