use std::collections::BTreeSet;
use std::mem;

use arctis_core::companion::adoption::{AdoptionStatus, DongleIdentity};
use arctis_core::companion::alerts::AlertId;
use arctis_core::companion::app_settings::{
    AppSettings, HeadsetValues, Persisted, PersistedChange, RoutePresets,
};
use arctis_core::companion::selection::PresetReason;
use arctis_core::companion::{Action, Companion, Input};
use arctis_core::eq::curve::{Curve, equivalent};
use arctis_core::eq::library;
use arctis_core::eq::preset::PresetId;
use arctis_core::intent::Intent;
use arctis_core::messages::audio::{
    AppIdentifiers, AudioEvent, DefaultSink, SinkKind, SoundPath, StreamInfo,
};
use arctis_core::messages::device::{Change, DeviceUpdate, Phase, SyncPolicy};
use arctis_core::protocol::{Command, Frame, Query, encode};
use arctis_core::session::timing::ENUMERATION_SETTLE;
use arctis_core::testkit::snapshots::{percent, values};
use arctis_core::testkit::{FakeHeadset, Flap, Simulation};
use arctis_core::time::Millis;

const START: Millis = Millis(1_760_000_000_000);
const STEP: u64 = 10;
const NOVA_5: u16 = 0x2232;
const CONNECTED: u64 = ENUMERATION_SETTLE + 2_000;

struct Stage {
    actions: Vec<Action>,
    wire: Vec<u8>,
    frames: Vec<Frame>,
    phases: Vec<Phase>,
}

struct EndToEnd {
    simulation: Simulation,
    companion: Companion,
    delivered: usize,
    wake: Option<Millis>,
    actions: Vec<Action>,
}

impl EndToEnd {
    fn new() -> Self {
        Self {
            simulation: Simulation::new(FakeHeadset::nova_5()),
            companion: Companion::new(persisted()),
            delivered: 0,
            wake: None,
            actions: Vec::new(),
        }
    }

    fn clock(&self) -> Millis {
        START.after(self.simulation.now().0)
    }

    fn send(&mut self, input: Input) {
        let actions = self.companion.handle(input, self.clock());
        for action in &actions {
            self.route(action);
        }
        self.actions.extend(actions);
    }

    fn route(&mut self, action: &Action) {
        match action {
            Action::SendDevice(Change::Desired { settings, policy }) => {
                self.simulation.desire(settings.clone(), *policy);
            }
            Action::WakeAt(at) => self.wake = Some(*at),
            _ => {}
        }
    }

    fn run_for(&mut self, duration: u64) {
        let end = self.simulation.now().after(duration);
        while self.simulation.now() < end {
            self.simulation.run_for(STEP);
            self.deliver_updates();
            self.wake_if_due();
        }
    }

    fn deliver_updates(&mut self) {
        let fresh: Vec<DeviceUpdate> = self.simulation.updates()[self.delivered..]
            .iter()
            .map(|(_, update)| update.clone())
            .collect();
        self.delivered += fresh.len();
        for update in fresh {
            self.send(Input::Device(update));
        }
    }

    fn wake_if_due(&mut self) {
        if self.wake.is_some_and(|at| at <= self.clock()) {
            self.wake = None;
            self.send(Input::Woke);
        }
    }

    fn stage(&mut self, act: impl FnOnce(&mut Self)) -> Stage {
        act(self);
        let stage = Stage {
            actions: mem::take(&mut self.actions),
            wire: self.simulation.sent_ids(),
            frames: self
                .simulation
                .writes()
                .iter()
                .map(|(_, frame)| frame.clone())
                .collect(),
            phases: self.simulation.published_phases(),
        };
        self.simulation.clear_logs();
        self.delivered = 0;
        stage
    }
}

fn id(text: &str) -> PresetId {
    PresetId::new(text)
}

fn curve(preset: &str) -> Curve {
    library::find(&id(preset))
        .expect("built-in preset")
        .curve()
        .clone()
}

fn persisted() -> Persisted {
    Persisted {
        settings: AppSettings {
            headset: values(),
            route_presets: RoutePresets {
                dongle: id("flat"),
                bluetooth: id("flat"),
            },
            app_rules: [("firefox".to_owned(), id("footsteps"))]
                .into_iter()
                .collect(),
            automatic_presets: true,
            adopted_identity: None,
            bluetooth_sink_seen: false,
            low_battery_threshold: percent(15),
        },
        custom_presets: Vec::new(),
        seen_apps: Vec::new(),
        battery: Vec::new(),
    }
}

fn firefox() -> StreamInfo {
    StreamInfo {
        ids: AppIdentifiers::new(Some("org.mozilla.firefox"), None, Some("firefox")),
        name: "Firefox".to_owned(),
        running: true,
        path: Some(SoundPath::Dongle),
    }
}

fn dongle_sink() -> DefaultSink {
    DefaultSink {
        terminal: SinkKind::NovaDongle,
        via_equalizer: false,
        via_easy_effects: false,
    }
}

fn equalizer_id() -> u8 {
    encode(&Command::SetEqualizer(curve("flat"))).payload()[0]
}

fn equalizer_frame(preset: &str) -> Frame {
    encode(&Command::SetEqualizer(curve(preset)))
}

fn connect_reads() -> Vec<u8> {
    [
        Query::Status,
        Query::Audio,
        Query::Ux,
        Query::Wireless,
        Query::Firmware,
    ]
    .into_iter()
    .map(Query::id)
    .collect()
}

const fn kind(action: &Action) -> &'static str {
    match action {
        Action::Persist(_) => "Persist",
        Action::StorePreset(_) => "StorePreset",
        Action::RecordBattery(_) => "RecordBattery",
        Action::RecordSeenApp(_) => "RecordSeenApp",
        Action::SendDevice(_) => "SendDevice",
        Action::SendAudio(_) => "SendAudio",
        Action::Publish(_) => "Publish",
        Action::Notify(_) => "Notify",
        Action::Withdraw(_) => "Withdraw",
        Action::WakeAt(_) => "WakeAt",
        Action::ReadyToQuit => "ReadyToQuit",
    }
}

fn kinds(actions: &[Action]) -> BTreeSet<&'static str> {
    actions.iter().map(kind).collect()
}

fn assert_only(actions: &[Action], allowed: &[&str]) {
    let unexpected: Vec<&str> = actions
        .iter()
        .map(kind)
        .filter(|name| !allowed.contains(name))
        .collect();
    assert!(unexpected.is_empty(), "unexpected actions: {unexpected:?}");
}

fn policies(actions: &[Action]) -> Vec<SyncPolicy> {
    actions
        .iter()
        .filter_map(|action| match action {
            Action::SendDevice(Change::Desired { policy, .. }) => Some(*policy),
            _ => None,
        })
        .collect()
}

fn curves(actions: &[Action]) -> Vec<Curve> {
    actions
        .iter()
        .filter_map(|action| match action {
            Action::SendDevice(Change::Desired { settings, .. }) => {
                Some(settings.equalizer.clone())
            }
            _ => None,
        })
        .collect()
}

fn battery_percents(actions: &[Action]) -> Vec<u8> {
    actions
        .iter()
        .filter_map(|action| match action {
            Action::RecordBattery(sample) if sample.reading.is_connected() => {
                Some(sample.reading.percent.get())
            }
            _ => None,
        })
        .collect()
}

fn alerts(actions: &[Action]) -> Vec<(bool, AlertId)> {
    actions
        .iter()
        .filter_map(|action| match action {
            Action::Notify(alert) => Some((true, alert.id)),
            Action::Withdraw(id) => Some((false, *id)),
            _ => None,
        })
        .collect()
}

fn connect(run: &mut EndToEnd) -> HeadsetValues {
    let connect = run.stage(|run| {
        run.simulation.plug();
        run.run_for(CONNECTED);
    });
    let factory = HeadsetValues::from_settings(run.simulation.headset().settings());
    assert_eq!(
        connect.phases,
        [
            Phase::Settling,
            Phase::Probing,
            Phase::Syncing,
            Phase::Ready
        ]
    );
    assert_eq!(connect.wire, connect_reads());
    assert_adopted(&connect.actions, factory);
    assert_eq!(
        run.companion.snapshot().dongle.adoption,
        AdoptionStatus::Adopted
    );
    factory
}

fn assert_adopted(actions: &[Action], factory: HeadsetValues) {
    let sent = policies(actions);
    assert_eq!(sent.first(), Some(&SyncPolicy::Adopt));
    assert_eq!(sent.last(), Some(&SyncPolicy::Enforce));
    let identity = DongleIdentity::new(NOVA_5, None);
    assert!(actions.contains(&Action::Persist(PersistedChange::AdoptedIdentity(identity))));
    assert!(actions.contains(&Action::Persist(PersistedChange::Headset(factory))));
    assert_eq!(battery_percents(actions), [37]);
    assert_only(
        actions,
        &["Persist", "SendDevice", "RecordBattery", "Publish"],
    );
}

fn set_route_preset(run: &mut EndToEnd) {
    let set = run.stage(|run| {
        run.send(Input::Intent(Intent::SetRoutePreset(
            SoundPath::Dongle,
            id("bass-boost"),
        )));
        run.run_for(1_000);
    });
    assert_eq!(set.wire, [equalizer_id(), Query::Audio.id()]);
    assert_eq!(set.frames.first(), Some(&equalizer_frame("bass-boost")));
    assert_eq!(curves(&set.actions), [curve("bass-boost")]);
    let persisted = PersistedChange::RoutePreset(SoundPath::Dongle, id("bass-boost"));
    assert!(set.actions.contains(&Action::Persist(persisted)));
    assert_only(&set.actions, &["Persist", "SendDevice", "Publish"]);
    assert!(equivalent(
        &run.simulation.headset().settings().equalizer,
        &curve("bass-boost")
    ));
}

fn switch_for_a_ruled_app(run: &mut EndToEnd) {
    let switch = run.stage(|run| {
        run.send(Input::Audio(AudioEvent::DefaultSink(dongle_sink())));
        run.send(Input::Audio(AudioEvent::Streams(vec![firefox()])));
        run.run_for(1_500);
    });
    assert_eq!(switch.wire, [equalizer_id(), Query::Audio.id()]);
    assert_eq!(switch.frames.first(), Some(&equalizer_frame("footsteps")));
    assert_eq!(curves(&switch.actions), [curve("footsteps")]);
    assert!(kinds(&switch.actions).is_superset(&BTreeSet::from(["WakeAt", "RecordSeenApp"])));
    assert_only(
        &switch.actions,
        &["WakeAt", "RecordSeenApp", "SendDevice", "Publish"],
    );
    let active = run.companion.snapshot().presets.active.dongle.clone();
    assert_eq!(active.preset, id("footsteps"));
    assert_eq!(active.reason, PresetReason::App("Firefox".to_owned()));
    assert!(equivalent(
        &run.simulation.headset().settings().equalizer,
        &curve("footsteps")
    ));
}

fn warn_about_a_low_battery(run: &mut EndToEnd) {
    let low = run.stage(|run| {
        run.simulation.battery(percent(14));
        run.run_for(200);
    });
    assert!(low.wire.is_empty());
    assert_eq!(alerts(&low.actions), [(true, AlertId::LowBattery)]);
    assert_eq!(battery_percents(&low.actions), [14]);
    assert_only(&low.actions, &["RecordBattery", "Notify", "Publish"]);
}

fn absorb_a_flap(run: &mut EndToEnd) {
    let flap = run.stage(|run| {
        run.simulation.flap(Flap {
            node_back_after: 300,
            ..Flap::default()
        });
        run.run_for(2_000);
    });
    assert!(flap.wire.is_empty(), "Rebinding sends no queries");
    assert!(flap.phases.is_empty(), "a short flap is silent");
    assert_eq!(run.simulation.phase(), Phase::Ready);
    assert_only(&flap.actions, &["Publish"]);
}

fn reconnect(run: &mut EndToEnd, factory: HeadsetValues) {
    let again = run.stage(|run| {
        run.simulation.unplug();
        run.run_for(500);
        run.simulation.plug();
        run.run_for(CONNECTED);
    });
    assert_eq!(
        again.phases,
        [
            Phase::Absent,
            Phase::Settling,
            Phase::Probing,
            Phase::Syncing,
            Phase::Ready
        ]
    );
    assert_eq!(
        again.wire,
        connect_reads(),
        "no drift, so nothing is written"
    );
    assert_reconnected(&again.actions);
    let settings = run.simulation.headset().settings();
    assert_eq!(HeadsetValues::from_settings(settings), factory);
    assert!(equivalent(&settings.equalizer, &curve("footsteps")));
}

fn assert_reconnected(actions: &[Action]) {
    let sent = policies(actions);
    assert!(!sent.contains(&SyncPolicy::Adopt));
    assert_eq!(sent.last(), Some(&SyncPolicy::Enforce));
    assert_eq!(
        alerts(actions),
        [(false, AlertId::LowBattery), (true, AlertId::LowBattery)]
    );
    assert_only(
        actions,
        &[
            "SendDevice",
            "RecordBattery",
            "Withdraw",
            "Notify",
            "Publish",
        ],
    );
}

#[test]
fn connect_adopt_set_switch_warn_flap_and_reconnect() {
    let mut run = EndToEnd::new();
    let factory = connect(&mut run);
    set_route_preset(&mut run);
    switch_for_a_ruled_app(&mut run);
    warn_about_a_low_battery(&mut run);
    absorb_a_flap(&mut run);
    reconnect(&mut run, factory);
}
