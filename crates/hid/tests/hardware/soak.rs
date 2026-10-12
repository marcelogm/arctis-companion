use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use arctis_core::messages::device::{Change, DeviceUpdate, Phase, SyncPolicy};
use arctis_core::settings::headset_settings::HeadsetSettings;
use arctis_core::settings::values::Sidetone;
use arctis_hid::port::{HidrawOpener, OpenError, Opener, Port};
use arctis_hid::waiter::PollWaiter;
use arctis_hid::worker::{Wiring, Worker};
use arctis_hid::{DeviceChanges, SysfsRoot, wake};

use crate::hardware_enabled;
use crate::soak_log::SoakLog;

const MINUTES: &str = "ARCTIS_SOAK_MINUTES";
const DEFAULT_MINUTES: u64 = 30;
const WRITE_EVERY: Duration = Duration::from_secs(60);
const RESTORE_WITHIN: Duration = Duration::from_secs(5);
const TICK: Duration = Duration::from_millis(50);
const REBINDS_PER_MINUTE: u64 = 3;

struct CountingOpener {
    opens: Arc<AtomicUsize>,
}

impl Opener for CountingOpener {
    fn open(&mut self, path: &Path) -> Result<Port, OpenError> {
        let opened = HidrawOpener.open(path);
        if opened.is_ok() {
            self.opens.fetch_add(1, Ordering::Relaxed);
        }
        opened
    }
}

struct Soak {
    started: Instant,
    changes: DeviceChanges,
    updates: async_channel::Receiver<DeviceUpdate>,
    log: SoakLog,
}

impl Soak {
    fn pump(&mut self) {
        while let Ok(update) = self.updates.try_recv() {
            self.log.record(self.started.elapsed(), update);
        }
    }

    fn run_for(&mut self, duration: Duration) {
        let end = Instant::now() + duration;
        while Instant::now() < end {
            self.pump();
            thread::sleep(TICK);
        }
    }

    fn write_toggle(&mut self) {
        let (Some(original), Some(observed)) = (&self.log.original, &self.log.observed) else {
            return;
        };
        if self.log.phase() != Some(Phase::Ready) || self.log.waiting_for.is_some() {
            return;
        }
        let desired = toggled(original, observed);
        self.log.requested += 1;
        self.log.waiting_for = Some(desired.clone());
        self.desire(desired);
    }

    fn restore(&mut self) {
        let Some(original) = self.log.original.clone() else {
            return;
        };
        if self.log.observed.as_ref() == Some(&original) {
            return;
        }
        self.log.requested += 1;
        self.log.waiting_for = Some(original.clone());
        self.desire(original);
        self.run_for(RESTORE_WITHIN);
    }

    fn desire(&self, settings: HeadsetSettings) {
        let change = Change::Desired {
            settings,
            policy: SyncPolicy::Enforce,
        };
        assert!(self.changes.send(change), "the device thread is gone");
    }
}

fn toggled(original: &HeadsetSettings, observed: &HeadsetSettings) -> HeadsetSettings {
    let level = original.sidetone.get();
    let other = if level == 0 { 1 } else { level - 1 };
    let next = if observed.sidetone.get() == level {
        other
    } else {
        level
    };
    HeadsetSettings {
        sidetone: Sidetone::try_from(next).expect("a sidetone level in range"),
        ..observed.clone()
    }
}

fn minutes() -> u64 {
    std::env::var(MINUTES)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_MINUTES)
}

#[test]
#[ignore = "needs the real dongle: ARCTIS_HW=1, runs ARCTIS_SOAK_MINUTES (30)"]
fn the_worker_rides_through_fwupd_rebinds_on_the_real_dongle() {
    if !hardware_enabled() {
        return;
    }
    let minutes = minutes();
    let opens = Arc::new(AtomicUsize::new(0));
    let (changes, mailbox) = wake::mailbox().expect("a mailbox");
    let (outbox, updates) = async_channel::unbounded();
    let wiring = Wiring {
        root: SysfsRoot::system(),
        opener: CountingOpener {
            opens: opens.clone(),
        },
        waiter: PollWaiter::new(),
        mailbox,
        updates: outbox,
    };
    let thread = thread::spawn(move || Worker::new(wiring).run());
    let mut soak = Soak {
        started: Instant::now(),
        changes,
        updates,
        log: SoakLog::default(),
    };
    for _ in 0..minutes * 60 / WRITE_EVERY.as_secs() {
        soak.write_toggle();
        soak.run_for(WRITE_EVERY);
    }
    soak.restore();
    let Soak { changes, log, .. } = soak;
    drop(changes);
    thread.join().expect("the device thread ends");
    report(minutes, opens.load(Ordering::Relaxed), &log);
}

fn report(minutes: u64, opens: usize, log: &SoakLog) {
    let rebinds = opens.saturating_sub(2);
    eprintln!(
        "soak: {minutes} min, {rebinds} rebinds ridden, phases {:?}, writes requested {}, confirmed {}, failed {}, interrupted by the headset turning off {}",
        log.phases, log.requested, log.confirmed, log.failed, log.interrupted
    );
    if log.requested == 0 {
        eprintln!("soak: the headset never reached Ready, so no write was exercised");
    }
    let expected = usize::try_from(minutes * REBINDS_PER_MINUTE / 2).unwrap_or(usize::MAX);
    assert!(rebinds >= expected, "too few rebinds: is fwupd running?");
    assert_eq!(log.disconnects(), [], "a user-visible disconnect");
    assert_eq!(log.failed, 0, "a write failed");
    assert_eq!(
        log.confirmed + log.interrupted,
        log.requested,
        "a write was lost"
    );
}
