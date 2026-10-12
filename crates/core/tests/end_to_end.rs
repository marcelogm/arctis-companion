use std::mem;

use arctis_core::companion::adoption::{AdoptionStatus, DongleIdentity};
use arctis_core::companion::alerts::AlertId;
use arctis_core::companion::app_settings::{HeadsetValues, PersistedChange};
use arctis_core::companion::selection::PresetReason;
use arctis_core::companion::{Action, Companion, Input};
use arctis_core::eq::curve::{Curve, equivalent};
use arctis_core::intent::Intent;
use arctis_core::messages::audio::{AudioCommand, AudioEvent, DefaultSink, SinkKind, SoundPath};
use arctis_core::messages::device::{Change, DeviceUpdate, Phase, SyncPolicy};
use arctis_core::protocol::{Command, Frame, Query, encode};
use arctis_core::session::timing::ENUMERATION_SETTLE;
use arctis_core::testkit::companion::{NOVA_5_PRODUCT, curve, firefox, with_rule};
use arctis_core::testkit::snapshots::{id, percent, values};
use arctis_core::testkit::{FakeHeadset, Flap, Simulation};
use arctis_core::time::Millis;

const START: Millis = Millis(1_760_000_000_000);
const STEP: u64 = 10;
const CONNECTED: u64 = ENUMERATION_SETTLE + 2_000;

struct Stage {
    steps: Vec<Step>,
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
    shown: Phase,
}

impl EndToEnd {
    fn new() -> Self {
        Self {
            simulation: Simulation::new(FakeHeadset::nova_5()),
            companion: Companion::new(with_rule("firefox", "footsteps")),
            delivered: 0,
            wake: None,
            actions: Vec::new(),
            shown: Phase::Absent,
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

    fn shown_phases(&mut self, actions: &[Action]) -> Vec<Phase> {
        let published = actions.iter().filter_map(|action| match action {
            Action::Publish(snapshot) => Some(snapshot.dongle.phase),
            _ => None,
        });
        let mut current = self.shown;
        let shown = published
            .filter(|phase| *phase != mem::replace(&mut current, *phase))
            .collect();
        self.shown = current;
        shown
    }

    fn stage(&mut self, act: impl FnOnce(&mut Self)) -> Stage {
        act(self);
        let actions = mem::take(&mut self.actions);
        let shown = self.shown_phases(&actions);
        let stage = Stage {
            steps: actions.iter().filter_map(step).collect(),
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
        assert_eq!(shown, stage.phases, "published phases follow the device");
        stage
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

#[derive(Debug, PartialEq)]
enum Step {
    Persist(PersistedChange),
    Desire(SyncPolicy, HeadsetValues, Curve),
    Battery(u8, bool),
    SeenApp(String),
    Audio(AudioCommand),
    Notify(AlertId),
    Withdraw(AlertId),
    WakeAfter(u64),
    Other(String),
}

fn step(action: &Action) -> Option<Step> {
    Some(match action {
        Action::Publish(_) => return None,
        Action::Persist(change) => Step::Persist(change.clone()),
        Action::SendDevice(Change::Desired { settings, policy }) => Step::Desire(
            *policy,
            HeadsetValues::from_settings(settings),
            settings.equalizer.clone(),
        ),
        Action::RecordBattery(sample) => {
            Step::Battery(sample.reading.percent.get(), sample.reading.is_connected())
        }
        Action::RecordSeenApp(app) => Step::SeenApp(app.key.clone()),
        Action::SendAudio(command) => Step::Audio(command.clone()),
        Action::Notify(alert) => Step::Notify(alert.id),
        Action::Withdraw(id) => Step::Withdraw(*id),
        Action::WakeAt(at) => Step::WakeAfter(at.since(START)),
        other => Step::Other(format!("{other:?}")),
    })
}

fn desire(policy: SyncPolicy, values: HeadsetValues, preset: &str) -> Step {
    Step::Desire(policy, values, curve(preset))
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
    let identity = DongleIdentity::new(NOVA_5_PRODUCT, None);
    assert_eq!(
        connect.steps,
        [
            desire(SyncPolicy::Adopt, values(), "flat"),
            Step::Battery(37, true),
            desire(SyncPolicy::Adopt, values(), "flat"),
            Step::Persist(PersistedChange::Headset(factory)),
            Step::Persist(PersistedChange::RoutePreset(SoundPath::Dongle, id("flat"))),
            Step::Persist(PersistedChange::AdoptedIdentity(identity)),
            desire(SyncPolicy::Enforce, factory, "flat"),
        ]
    );
    assert_eq!(
        run.companion.snapshot().dongle.adoption,
        AdoptionStatus::Adopted
    );
    factory
}

fn set_route_preset(run: &mut EndToEnd, factory: HeadsetValues) {
    let set = run.stage(|run| {
        run.send(Input::Intent(Intent::SetRoutePreset(
            SoundPath::Dongle,
            id("bass-boost"),
        )));
        run.run_for(1_000);
    });
    assert_eq!(set.wire, [equalizer_id(), Query::Audio.id()]);
    assert_eq!(set.frames.first(), Some(&equalizer_frame("bass-boost")));
    let persisted = PersistedChange::RoutePreset(SoundPath::Dongle, id("bass-boost"));
    assert_eq!(
        set.steps,
        [
            Step::Persist(persisted),
            desire(SyncPolicy::Enforce, factory, "bass-boost"),
        ]
    );
    assert!(equivalent(
        &run.simulation.headset().settings().equalizer,
        &curve("bass-boost")
    ));
}

fn switch_for_a_ruled_app(run: &mut EndToEnd, factory: HeadsetValues) {
    let switch = run.stage(|run| {
        run.send(Input::Audio(AudioEvent::DefaultSink(dongle_sink())));
        let firefox = firefox(Some(SoundPath::Dongle));
        run.send(Input::Audio(AudioEvent::Streams(vec![firefox])));
        run.run_for(1_500);
    });
    assert_eq!(switch.wire, [equalizer_id(), Query::Audio.id()]);
    assert_eq!(switch.frames.first(), Some(&equalizer_frame("footsteps")));
    assert_eq!(
        switch.steps,
        [
            Step::SeenApp("org.mozilla.firefox".to_owned()),
            Step::WakeAfter(7_000),
            desire(SyncPolicy::Enforce, factory, "footsteps"),
        ]
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
    assert_eq!(
        low.steps,
        [Step::Battery(14, true), Step::Notify(AlertId::LowBattery)]
    );
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
    assert_eq!(flap.steps, []);
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
    assert_eq!(
        again.steps,
        [
            Step::Battery(14, false),
            Step::Withdraw(AlertId::LowBattery),
            desire(SyncPolicy::Enforce, factory, "footsteps"),
            Step::Battery(14, true),
            Step::Notify(AlertId::LowBattery),
            desire(SyncPolicy::Enforce, factory, "footsteps"),
        ]
    );
    let settings = run.simulation.headset().settings();
    assert_eq!(HeadsetValues::from_settings(settings), factory);
    assert!(equivalent(&settings.equalizer, &curve("footsteps")));
}

#[test]
fn connect_adopt_set_switch_warn_flap_and_reconnect() {
    let mut run = EndToEnd::new();
    let factory = connect(&mut run);
    set_route_preset(&mut run, factory);
    switch_for_a_ruled_app(&mut run, factory);
    warn_about_a_low_battery(&mut run);
    absorb_a_flap(&mut run);
    reconnect(&mut run, factory);
}
