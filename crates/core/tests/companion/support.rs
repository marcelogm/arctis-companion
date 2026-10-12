use arctis_core::eq::library;
use arctis_core::eq::preset::{Preset, PresetId};
use arctis_core::eq::{Curve, Filter, FilterKind};
use arctis_core::messages::audio::{AppIdentifiers, SoundPath, StreamInfo};
use arctis_core::messages::audio::{DefaultSink, SinkKind};
use arctis_core::settings::headset_settings::HeadsetSettings;
use arctis_core::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};

pub fn sink(terminal: SinkKind, via_equalizer: bool, via_easy_effects: bool) -> DefaultSink {
    DefaultSink {
        terminal,
        via_equalizer,
        via_easy_effects,
    }
}

pub fn id(text: &str) -> PresetId {
    PresetId::new(text)
}

pub fn curve_of(preset: &str) -> Curve {
    library::find(&id(preset))
        .expect("built-in preset")
        .curve()
        .clone()
}

pub fn unique_curve() -> Curve {
    let filter = Filter::from_parts(FilterKind::Peaking, 1_000, 12, 1_000).expect("filter");
    Curve::from_array([filter])
}

pub fn custom(preset: &str, curve: Curve) -> Preset {
    Preset::custom(id(preset), preset.to_uppercase(), curve)
}

pub fn stream(app: &str, path: Option<SoundPath>, running: bool) -> StreamInfo {
    StreamInfo {
        ids: AppIdentifiers::new(Some(&format!("org.{app}.App")), None, Some(app)),
        name: app.to_owned(),
        running,
        path,
    }
}

pub fn observed_with(equalizer: Curve) -> HeadsetSettings {
    HeadsetSettings {
        mic_volume: MicVolume::try_from(7).expect("mic volume"),
        sidetone: Sidetone::try_from(3).expect("sidetone"),
        mute_light: MuteLight::Low,
        volume_limiter: false,
        wireless_mode: WirelessMode::ExtendedRange,
        auto_power_off: AutoPowerOff::try_from(15).expect("auto power off"),
        equalizer,
    }
}

pub fn observed() -> HeadsetSettings {
    observed_with(curve_of("flat"))
}
