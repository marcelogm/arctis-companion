use arctis_core::eq::Curve;
use arctis_core::messages::audio::{
    AppIdentifiers, AudioCommand, AudioEvent, DefaultSink, SinkKind, SoundPath, StreamInfo,
};
use arctis_core::messages::device::{Change, DeviceUpdate, Phase, SyncPolicy};
use arctis_core::protocol::firmware::{Firmware, Version};
use arctis_core::protocol::model::{Model, ModelLookup};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::headset_settings::{HeadsetSettings, Setting, SettingChange};
use arctis_core::settings::values::{
    AutoPowerOff, MicVolume, MuteLight, Percent, Sidetone, WirelessMode,
};

fn settings() -> HeadsetSettings {
    HeadsetSettings {
        mic_volume: MicVolume::try_from(15).expect("mic volume"),
        sidetone: Sidetone::try_from(0).expect("sidetone"),
        mute_light: MuteLight::High,
        volume_limiter: true,
        wireless_mode: WirelessMode::Standard,
        auto_power_off: AutoPowerOff::try_from(30).expect("auto power off"),
        equalizer: Curve::default(),
    }
}

fn nova_5() -> Model {
    let ModelLookup::Supported(model) = Model::lookup(0x2232) else {
        panic!("the Nova 5 is supported");
    };
    model
}

fn all_device_updates() -> Vec<DeviceUpdate> {
    let version = Version {
        major: 1,
        minor: 16,
        patch: 0,
    };
    vec![
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Identity {
            model: nova_5(),
            uniq: None,
            firmware: Firmware {
                receiver: Some(version),
                headset: None,
            },
        },
        DeviceUpdate::Observed(settings()),
        DeviceUpdate::Status {
            battery: Percent::try_from(73).expect("percent"),
            charging: Charging::Discharging,
            link: Link::Connected,
        },
        DeviceUpdate::HeadsetChanged(SettingChange::MuteLight(MuteLight::Low)),
        DeviceUpdate::WriteFailed(Setting::Sidetone),
    ]
}

const fn device_update_name(update: &DeviceUpdate) -> &'static str {
    match update {
        DeviceUpdate::Phase(_) => "Phase",
        DeviceUpdate::Identity { .. } => "Identity",
        DeviceUpdate::Observed(_) => "Observed",
        DeviceUpdate::Status { .. } => "Status",
        DeviceUpdate::HeadsetChanged(_) => "HeadsetChanged",
        DeviceUpdate::WriteFailed(_) => "WriteFailed",
    }
}

const fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::Absent => "Absent",
        Phase::PermissionDenied => "PermissionDenied",
        Phase::Settling => "Settling",
        Phase::Probing => "Probing",
        Phase::HeadsetOff => "HeadsetOff",
        Phase::Syncing => "Syncing",
        Phase::Ready => "Ready",
        Phase::Rebinding => "Rebinding",
    }
}

fn stream() -> StreamInfo {
    StreamInfo {
        ids: AppIdentifiers::new(Some("org.mozilla.firefox"), None, Some("firefox")),
        name: "Firefox".to_owned(),
        running: true,
        path: Some(SoundPath::Dongle),
    }
}

fn all_audio_events() -> Vec<AudioEvent> {
    vec![
        AudioEvent::Available,
        AudioEvent::Unavailable,
        AudioEvent::DefaultSink(DefaultSink {
            terminal: SinkKind::NovaBluetooth,
            via_equalizer: true,
            via_easy_effects: false,
        }),
        AudioEvent::Streams(vec![stream()]),
        AudioEvent::ManagerDenied,
        AudioEvent::BluetoothNodeSeen,
    ]
}

const fn audio_event_name(event: &AudioEvent) -> &'static str {
    match event {
        AudioEvent::Available => "Available",
        AudioEvent::Unavailable => "Unavailable",
        AudioEvent::DefaultSink(_) => "DefaultSink",
        AudioEvent::Streams(_) => "Streams",
        AudioEvent::ManagerDenied => "ManagerDenied",
        AudioEvent::BluetoothNodeSeen => "BluetoothNodeSeen",
    }
}

const fn audio_command_name(command: &AudioCommand) -> &'static str {
    match command {
        AudioCommand::InsertEqualizer(_) => "InsertEqualizer",
        AudioCommand::Retarget(_) => "Retarget",
        AudioCommand::RemoveEqualizer => "RemoveEqualizer",
    }
}

const fn change_policy(change: &Change) -> SyncPolicy {
    match change {
        Change::Desired { policy, .. } => *policy,
    }
}

fn crosses_threads<T: Send + 'static>() {}

#[test]
fn device_updates_are_the_six_of_device_section_6() {
    let names: Vec<&str> = all_device_updates()
        .iter()
        .map(device_update_name)
        .collect();
    assert_eq!(
        names,
        [
            "Phase",
            "Identity",
            "Observed",
            "Status",
            "HeadsetChanged",
            "WriteFailed"
        ]
    );
}

#[test]
fn phases_are_the_eight_of_the_glossary() {
    let phases = [
        Phase::Absent,
        Phase::PermissionDenied,
        Phase::Settling,
        Phase::Probing,
        Phase::HeadsetOff,
        Phase::Syncing,
        Phase::Ready,
        Phase::Rebinding,
    ];
    let names: Vec<&str> = phases.into_iter().map(phase_name).collect();
    assert_eq!(
        names,
        [
            "Absent",
            "PermissionDenied",
            "Settling",
            "Probing",
            "HeadsetOff",
            "Syncing",
            "Ready",
            "Rebinding",
        ]
    );
}

#[test]
fn a_desired_change_carries_settings_and_policy_together() {
    for policy in [SyncPolicy::Adopt, SyncPolicy::Enforce] {
        let change = Change::Desired {
            settings: settings(),
            policy,
        };
        assert_eq!(change_policy(&change), policy);
    }
}

#[test]
fn audio_events_are_the_six_of_design_section_3_2() {
    let names: Vec<&str> = all_audio_events().iter().map(audio_event_name).collect();
    assert_eq!(
        names,
        [
            "Available",
            "Unavailable",
            "DefaultSink",
            "Streams",
            "ManagerDenied",
            "BluetoothNodeSeen"
        ]
    );
}

#[test]
fn audio_commands_are_the_three_of_design_section_3_2() {
    let curve = Curve::default();
    let commands = [
        AudioCommand::InsertEqualizer(curve.clone()),
        AudioCommand::Retarget(curve),
        AudioCommand::RemoveEqualizer,
    ];
    let names: Vec<&str> = commands.iter().map(audio_command_name).collect();
    assert_eq!(names, ["InsertEqualizer", "Retarget", "RemoveEqualizer"]);
}

#[test]
fn every_message_can_cross_threads() {
    crosses_threads::<Change>();
    crosses_threads::<DeviceUpdate>();
    crosses_threads::<AudioCommand>();
    crosses_threads::<AudioEvent>();
}

#[test]
fn app_identifiers_keep_their_order_and_the_first_is_the_key() {
    let ids = AppIdentifiers::new(Some("org.mozilla.firefox"), None, Some("Firefox"));
    assert_eq!(ids.as_slice(), ["org.mozilla.firefox", "firefox"]);
    assert_eq!(ids.key(), Some("org.mozilla.firefox"));
}

#[test]
fn app_identifiers_skip_empty_and_repeated_values() {
    let ids = AppIdentifiers::new(Some(""), Some("firefox"), Some("FIREFOX"));
    assert_eq!(ids.as_slice(), ["firefox"]);
}

#[test]
fn a_stream_without_identifiers_has_no_key() {
    assert_eq!(AppIdentifiers::new(None, None, None).key(), None);
}
