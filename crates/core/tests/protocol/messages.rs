use arctis_core::eq::Curve;
use arctis_core::messages::audio::{
    AppIdentifiers, AudioCommand, AudioEvent, DefaultSink, SinkKind, SoundPath, StreamInfo,
};
use arctis_core::messages::device::{Change, DeviceUpdate, Phase, SyncPolicy};
use arctis_core::protocol::firmware::Firmware;
use arctis_core::protocol::model::{Model, ModelLookup};
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::settings::headset_settings::{HeadsetSettings, Setting, SettingChange};
use arctis_core::settings::values::{MuteLight, Percent};

use crate::support::{percent, sample_settings, version};

const fn has_type<T>(_: &T) {}

fn crosses_threads<T: Send + 'static>() {}

fn nova_5() -> Model {
    let ModelLookup::Supported(model) = Model::lookup(0x2232) else {
        panic!("the Nova 5 is supported");
    };
    model
}

fn device_updates() -> [DeviceUpdate; 6] {
    [
        DeviceUpdate::Phase(Phase::Ready),
        DeviceUpdate::Identity {
            model: nova_5(),
            uniq: None,
            firmware: Firmware {
                receiver: Some(version(1, 16, 0)),
                headset: None,
            },
        },
        DeviceUpdate::Observed(sample_settings()),
        DeviceUpdate::Status {
            battery: percent(73),
            charging: Charging::Discharging,
            link: Link::Connected,
        },
        DeviceUpdate::HeadsetChanged(SettingChange::MuteLight(MuteLight::Low)),
        DeviceUpdate::WriteFailed(Setting::Sidetone),
    ]
}

fn device_update_shape(update: &DeviceUpdate) {
    match update {
        DeviceUpdate::Phase(phase) => has_type::<Phase>(phase),
        DeviceUpdate::Identity {
            model,
            uniq,
            firmware,
        } => {
            has_type::<Model>(model);
            has_type::<Option<String>>(uniq);
            has_type::<Firmware>(firmware);
        }
        DeviceUpdate::Observed(settings) => has_type::<HeadsetSettings>(settings),
        DeviceUpdate::Status {
            battery,
            charging,
            link,
        } => {
            has_type::<Percent>(battery);
            has_type::<Charging>(charging);
            has_type::<Link>(link);
        }
        DeviceUpdate::HeadsetChanged(change) => has_type::<SettingChange>(change),
        DeviceUpdate::WriteFailed(setting) => has_type::<Setting>(setting),
    }
}

const fn phase_shape(phase: Phase) {
    match phase {
        Phase::Absent
        | Phase::PermissionDenied
        | Phase::Settling
        | Phase::Probing
        | Phase::HeadsetOff
        | Phase::Syncing
        | Phase::Ready
        | Phase::Rebinding => {}
    }
}

fn change_shape(change: &Change) {
    let Change::Desired { settings, policy } = change;
    has_type::<HeadsetSettings>(settings);
    match policy {
        SyncPolicy::Adopt | SyncPolicy::Enforce => {}
    }
}

fn audio_events() -> [AudioEvent; 7] {
    let stream = StreamInfo {
        ids: AppIdentifiers::new(Some("org.mozilla.firefox"), None, Some("firefox")),
        name: "Firefox".to_owned(),
        running: true,
        path: Some(SoundPath::Dongle),
    };
    [
        AudioEvent::Available,
        AudioEvent::Unavailable,
        AudioEvent::DefaultSink(DefaultSink {
            terminal: SinkKind::NovaBluetooth,
            via_equalizer: true,
            via_easy_effects: false,
        }),
        AudioEvent::Streams(vec![stream]),
        AudioEvent::ManagerDenied,
        AudioEvent::BluetoothNodeSeen,
        AudioEvent::EqualizerNotInserted,
    ]
}

fn audio_event_shape(event: &AudioEvent) {
    match event {
        AudioEvent::Available
        | AudioEvent::Unavailable
        | AudioEvent::ManagerDenied
        | AudioEvent::BluetoothNodeSeen
        | AudioEvent::EqualizerNotInserted => {}
        AudioEvent::DefaultSink(sink) => has_type::<DefaultSink>(sink),
        AudioEvent::Streams(streams) => has_type::<Vec<StreamInfo>>(streams),
    }
}

fn audio_command_shape(command: &AudioCommand) {
    match command {
        AudioCommand::InsertEqualizer(curve) | AudioCommand::Retarget(curve) => {
            has_type::<Curve>(curve);
        }
        AudioCommand::RemoveEqualizer => {}
    }
}

#[test]
fn device_update_variants_match_device_section_6() {
    device_updates().iter().for_each(device_update_shape);
}

#[test]
fn phase_variants_match_the_glossary() {
    [
        Phase::Absent,
        Phase::PermissionDenied,
        Phase::Settling,
        Phase::Probing,
        Phase::HeadsetOff,
        Phase::Syncing,
        Phase::Ready,
        Phase::Rebinding,
    ]
    .into_iter()
    .for_each(phase_shape);
}

#[test]
fn change_variants_match_design_section_3_2() {
    for policy in [SyncPolicy::Adopt, SyncPolicy::Enforce] {
        change_shape(&Change::Desired {
            settings: sample_settings(),
            policy,
        });
    }
}

#[test]
fn audio_event_variants_match_design_section_3_2() {
    audio_events().iter().for_each(audio_event_shape);
}

#[test]
fn audio_command_variants_match_design_section_3_2() {
    [
        AudioCommand::InsertEqualizer(Curve::default()),
        AudioCommand::Retarget(Curve::default()),
        AudioCommand::RemoveEqualizer,
    ]
    .iter()
    .for_each(audio_command_shape);
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
