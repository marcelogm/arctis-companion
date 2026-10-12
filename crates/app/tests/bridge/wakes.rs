use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use arctis_app::alarm::{GlibAlarm, SystemClock};
use arctis_app::ports::{Alarm, Clock};
use arctis_core::companion::Input;
use arctis_core::messages::audio::{AudioCommand, AudioEvent, DefaultSink, SinkKind, SoundPath};
use gtk::glib;

use crate::fakes::Seen;
use crate::rig::{Rig, curve, firefox, with_rule};

fn bluetooth_rig() -> Rig {
    let rig = Rig::new(with_rule("firefox", "bass-boost"));
    let sink = DefaultSink {
        terminal: SinkKind::NovaBluetooth,
        via_equalizer: false,
        via_easy_effects: false,
    };
    let _ = rig.send(Input::Audio(AudioEvent::DefaultSink(sink)));
    rig
}

#[test]
fn a_wake_at_schedules_the_alarm_for_its_delay() {
    let rig = bluetooth_rig();
    let streams = AudioEvent::Streams(vec![firefox(Some(SoundPath::Bluetooth))]);
    let seen = rig.send(Input::Audio(streams));
    assert!(seen.contains(&Seen::Scheduled(Duration::from_secs(1))));
}

#[test]
fn the_alarm_delivers_woke_to_the_companion() {
    let rig = bluetooth_rig();
    let _ = rig.send(Input::Audio(AudioEvent::Streams(vec![firefox(Some(
        SoundPath::Bluetooth,
    ))])));
    rig.clock.advance(1_000);
    rig.alarm.fire();
    let seen = rig.take();
    assert!(seen.contains(&Seen::Audio(AudioCommand::Retarget(curve("bass-boost")))));
}

#[test]
fn the_glib_alarm_keeps_only_the_latest_wake() {
    let context = glib::MainContext::new();
    context
        .with_thread_default(|| {
            let alarm = GlibAlarm::default();
            let fired = Rc::new(RefCell::new(Vec::new()));
            let (first, second) = (Rc::clone(&fired), Rc::clone(&fired));
            alarm.schedule(
                Duration::from_millis(30),
                Box::new(move || first.borrow_mut().push("first")),
            );
            alarm.schedule(
                Duration::from_millis(5),
                Box::new(move || second.borrow_mut().push("second")),
            );
            let deadline = Instant::now() + Duration::from_millis(200);
            while Instant::now() < deadline {
                context.iteration(false);
                thread::sleep(Duration::from_millis(1));
            }
            assert_eq!(*fired.borrow(), ["second"]);
        })
        .expect("the test owns its main context");
}

#[test]
fn the_system_clock_counts_wall_clock_milliseconds() {
    let unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("after 1970")
        .as_millis();
    let now = u128::from(SystemClock.now().0);
    assert!(now.abs_diff(unix) < 1_000);
}
