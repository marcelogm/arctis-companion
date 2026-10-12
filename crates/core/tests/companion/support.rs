use arctis_core::battery::sample::Reading;
use arctis_core::companion::adoption::DongleIdentity;
use arctis_core::companion::app_settings::PersistedChange;
use arctis_core::companion::app_settings::{
    AppRules, AppSettings, HeadsetValues, Persisted, RoutePresets,
};
use arctis_core::companion::selection::{ActivePreset, PresetReason};
use arctis_core::companion::{Action, Companion, Input, Snapshot};
use arctis_core::eq::library;
use arctis_core::eq::preset::{Preset, PresetId};
use arctis_core::eq::{Curve, Filter, FilterKind};
use arctis_core::intent::Intent;
use arctis_core::messages::audio::{AppIdentifiers, SoundPath, StreamInfo};
use arctis_core::messages::audio::{AudioCommand, AudioEvent};
use arctis_core::messages::audio::{DefaultSink, SinkKind};
use arctis_core::messages::device::{Change, SyncPolicy};
use arctis_core::messages::device::{DeviceUpdate, Phase};
use arctis_core::protocol::firmware::{Firmware, Version};
use arctis_core::protocol::model::{Model, ModelLookup};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::headset_settings::HeadsetSettings;
use arctis_core::settings::values::Percent;
use arctis_core::settings::values::{AutoPowerOff, MicVolume, MuteLight, Sidetone, WirelessMode};
use arctis_core::time::Millis;
use std::rc::Rc;

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

pub fn percent(value: u8) -> Percent {
    Percent::try_from(value).expect("percent")
}

pub fn reading(value: u8, charging: Charging, link: Link) -> Reading {
    Reading {
        percent: percent(value),
        charging,
        link,
    }
}

pub fn discharging(value: u8) -> Reading {
    reading(value, Charging::Discharging, Link::Connected)
}

pub const NOVA_5: u16 = 0x2232;

pub fn nova_5() -> Model {
    let ModelLookup::Supported(model) = Model::lookup(NOVA_5) else {
        panic!("the Nova 5 is supported");
    };
    model
}

pub const fn firmware(minor: u8) -> Firmware {
    Firmware {
        receiver: Some(Version {
            major: 1,
            minor,
            patch: 0,
        }),
        headset: None,
    }
}

pub const START: Millis = Millis(1_760_000_000_000);

pub fn defaults() -> HeadsetValues {
    HeadsetValues {
        mic_volume: MicVolume::try_from(10).expect("mic volume"),
        sidetone: Sidetone::try_from(0).expect("sidetone"),
        mute_light: MuteLight::Medium,
        volume_limiter: true,
        wireless_mode: WirelessMode::Standard,
        auto_power_off: AutoPowerOff::try_from(30).expect("auto power off"),
    }
}

pub fn settings() -> AppSettings {
    AppSettings {
        headset: defaults(),
        route_presets: RoutePresets {
            dongle: id("flat"),
            bluetooth: id("flat"),
        },
        app_rules: AppRules::default(),
        automatic_presets: true,
        adopted_identity: None,
        bluetooth_sink_seen: false,
        low_battery_threshold: percent(15),
    }
}

pub fn persisted() -> Persisted {
    Persisted {
        settings: settings(),
        custom_presets: Vec::new(),
        seen_apps: Vec::new(),
        battery: Vec::new(),
    }
}

pub fn adopted() -> Persisted {
    let mut saved = persisted();
    saved.settings.adopted_identity = Some(DongleIdentity::new(NOVA_5, None));
    saved.settings.headset = HeadsetValues::from_settings(&observed());
    saved
}

pub struct Harness {
    pub companion: Companion,
    pub now: Millis,
}

impl Harness {
    pub fn new(persisted: Persisted) -> Self {
        Self {
            companion: Companion::new(persisted),
            now: START,
        }
    }

    pub fn at(&mut self, offset: u64) -> &mut Self {
        self.now = START.after(offset);
        self
    }

    pub fn send(&mut self, input: Input) -> Vec<Action> {
        self.companion.handle(input, self.now)
    }

    pub fn device(&mut self, update: DeviceUpdate) -> Vec<Action> {
        self.send(Input::Device(update))
    }

    pub fn audio(&mut self, event: AudioEvent) -> Vec<Action> {
        self.send(Input::Audio(event))
    }

    pub fn intent(&mut self, intent: Intent) -> Vec<Action> {
        self.send(Input::Intent(intent))
    }

    pub fn headset_holds(&mut self, preset: &str) -> Vec<Action> {
        self.device(DeviceUpdate::Observed(observed_with(curve_of(preset))))
    }

    pub fn connect_as(
        &mut self,
        uniq: Option<&str>,
        minor: u8,
        observed: HeadsetSettings,
    ) -> Vec<Action> {
        let identity = DeviceUpdate::Identity {
            model: nova_5(),
            uniq: uniq.map(str::to_owned),
            firmware: firmware(minor),
        };
        [
            DeviceUpdate::Phase(Phase::Settling),
            identity,
            DeviceUpdate::Phase(Phase::Syncing),
            DeviceUpdate::Observed(observed),
            DeviceUpdate::Phase(Phase::Ready),
        ]
        .into_iter()
        .flat_map(|update| self.device(update))
        .collect()
    }

    pub fn connect(&mut self, observed: HeadsetSettings) -> Vec<Action> {
        self.connect_as(None, 16, observed)
    }

    pub fn snapshot(&self) -> Rc<Snapshot> {
        self.companion.snapshot()
    }
}

pub fn published(actions: &[Action]) -> Option<Rc<Snapshot>> {
    actions.iter().rev().find_map(|action| match action {
        Action::Publish(snapshot) => Some(Rc::clone(snapshot)),
        _ => None,
    })
}

pub fn wakes(actions: &[Action]) -> Vec<Millis> {
    actions
        .iter()
        .filter_map(|action| match action {
            Action::WakeAt(at) => Some(*at),
            _ => None,
        })
        .collect()
}

pub fn device_changes(actions: &[Action]) -> Vec<(HeadsetSettings, SyncPolicy)> {
    actions
        .iter()
        .filter_map(|action| match action {
            Action::SendDevice(Change::Desired { settings, policy }) => {
                Some((settings.clone(), *policy))
            }
            _ => None,
        })
        .collect()
}

pub fn persisted_changes(actions: &[Action]) -> Vec<PersistedChange> {
    actions
        .iter()
        .filter_map(|action| match action {
            Action::Persist(change) => Some(change.clone()),
            _ => None,
        })
        .collect()
}

pub fn audio_commands(actions: &[Action]) -> Vec<AudioCommand> {
    actions
        .iter()
        .filter_map(|action| match action {
            Action::SendAudio(command) => Some(command.clone()),
            _ => None,
        })
        .collect()
}

pub fn default_sink(terminal: SinkKind) -> AudioEvent {
    AudioEvent::DefaultSink(sink(terminal, false, false))
}

pub fn device_curves(actions: &[Action]) -> Vec<Curve> {
    device_changes(actions)
        .into_iter()
        .map(|(settings, _)| settings.equalizer)
        .collect()
}

pub fn rules(entries: &[(&str, &str)]) -> AppRules {
    entries
        .iter()
        .map(|(app, preset)| ((*app).to_owned(), id(preset)))
        .collect()
}

pub fn with_rules(entries: &[(&str, &str)]) -> Persisted {
    let mut saved = adopted();
    saved.settings.app_rules = rules(entries);
    saved
}

pub fn ruled(entries: &[(&str, &str)]) -> Harness {
    let mut h = Harness::new(with_rules(entries));
    let _ = h.connect(observed());
    h
}

pub fn playing(apps: &[&str], path: SoundPath) -> AudioEvent {
    AudioEvent::Streams(
        apps.iter()
            .map(|app| stream(app, Some(path), true))
            .collect(),
    )
}

pub fn chosen_for(preset: &str, app: &str) -> ActivePreset {
    ActivePreset {
        preset: id(preset),
        reason: PresetReason::App(app.to_owned()),
    }
}

pub fn ready() -> Harness {
    let mut h = Harness::new(adopted());
    let _ = h.connect(observed());
    h
}

pub fn firefox_playing() -> Harness {
    let mut h = ruled(&[("firefox", "footsteps"), ("spotify", "bass-boost")]);
    let _ = h.at(1_000).audio(playing(&["firefox"], SoundPath::Dongle));
    let _ = h.at(2_000).send(Input::Woke);
    let _ = h.headset_holds("footsteps");
    h
}

pub fn picked_by_hand(preset: &str, app: Option<&str>) -> ActivePreset {
    ActivePreset {
        preset: id(preset),
        reason: PresetReason::Manual(app.map(str::to_owned)),
    }
}
