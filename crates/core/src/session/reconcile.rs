use crate::eq::curve::equivalent;
use crate::protocol::{Command, Query, Reply};
use crate::settings::headset_settings::{HeadsetSettings, Setting, SettingChange};

const WRITE_ORDER: [Setting; 7] = [
    Setting::MicVolume,
    Setting::Sidetone,
    Setting::MuteLight,
    Setting::VolumeLimiter,
    Setting::AutoPowerOff,
    Setting::Equalizer,
    Setting::WirelessMode,
];

#[must_use]
pub(crate) fn drift(
    desired: &HeadsetSettings,
    observed: &HeadsetSettings,
    skipped: &[Setting],
) -> Vec<SettingChange> {
    WRITE_ORDER
        .into_iter()
        .filter(|setting| !skipped.contains(setting))
        .map(|setting| desired.value_of(setting))
        .filter(|wanted| !holds(observed, wanted))
        .collect()
}

#[must_use]
pub(crate) fn holds(settings: &HeadsetSettings, wanted: &SettingChange) -> bool {
    same(&settings.value_of(wanted.setting()), wanted)
}

#[must_use]
pub(crate) fn same(a: &SettingChange, b: &SettingChange) -> bool {
    match (a, b) {
        (SettingChange::Equalizer(left), SettingChange::Equalizer(right)) => {
            equivalent(left, right)
        }
        _ => a == b,
    }
}

#[must_use]
pub(crate) fn readbacks(changes: &[SettingChange]) -> Vec<Query> {
    let mut queries = Vec::new();
    for query in changes
        .iter()
        .filter_map(|change| Command::from(change.clone()).readback())
    {
        if !queries.contains(&query) {
            queries.push(query);
        }
    }
    queries
}

#[must_use]
pub(crate) fn readings(reply: &Reply) -> Vec<SettingChange> {
    match reply {
        Reply::Audio(audio) => vec![
            SettingChange::MicVolume(audio.mic_volume),
            SettingChange::Sidetone(audio.sidetone),
            SettingChange::VolumeLimiter(audio.volume_limiter),
            SettingChange::Equalizer(audio.equalizer.clone()),
        ],
        Reply::Ux(ux) => vec![
            SettingChange::MuteLight(ux.mute_light),
            SettingChange::AutoPowerOff(ux.auto_power_off),
        ],
        Reply::Wireless(mode) => vec![SettingChange::WirelessMode(*mode)],
        Reply::Status(_) | Reply::Firmware(_) => Vec::new(),
    }
}
