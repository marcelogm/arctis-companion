use std::cell::RefCell;
use std::os::fd::BorrowedFd;
use std::path::Path;
use std::rc::Rc;

use arctis_core::messages::device::{Change, DeviceUpdate, Phase, SyncPolicy};
use arctis_core::protocol::Frame;
use arctis_core::protocol::status::Link;
use arctis_core::settings::headset_settings::{HeadsetSettings, SettingChange};
use arctis_core::settings::values::Percent;
use arctis_core::testkit::{FakeHeadset, Flap, Rig};
use arctis_core::time::Millis;
use arctis_hid::DeviceChanges;
use arctis_hid::port::{OpenError, Opener, Port};
use arctis_hid::waiter::Waiter;
use arctis_hid::wake;
use arctis_hid::worker::{Wiring, Worker};
use rustix::event::{PollFd, PollFlags, Timespec};

use crate::bench::{Advance, Bench};

const TURN_LIMIT: usize = 200_000;

type Shared = Rc<RefCell<Bench>>;

pub struct FakeOpener(Shared);

impl Opener for FakeOpener {
    fn open(&mut self, path: &Path) -> Result<Port, OpenError> {
        self.0.borrow_mut().open(path)
    }
}

pub struct VirtualWaiter(Shared);

impl Waiter for VirtualWaiter {
    fn now(&self) -> Millis {
        self.0.borrow().now
    }

    fn wait(&mut self, fds: &[BorrowedFd<'_>], deadline: Option<Millis>) -> Vec<bool> {
        wait_on(&mut self.0.borrow_mut(), fds, deadline)
    }
}

fn wait_on(bench: &mut Bench, fds: &[BorrowedFd<'_>], deadline: Option<Millis>) -> Vec<bool> {
    loop {
        bench.collect_writes();
        let ready = ready_now(fds);
        if ready.contains(&true) || bench.advance(deadline) == Advance::TimedOut {
            return ready;
        }
    }
}

fn ready_now(fds: &[BorrowedFd<'_>]) -> Vec<bool> {
    let mut polled: Vec<PollFd<'_>> = fds
        .iter()
        .map(|fd| PollFd::from_borrowed_fd(*fd, PollFlags::IN))
        .collect();
    let zero = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    rustix::event::poll(&mut polled, Some(&zero)).expect("poll");
    polled.iter().map(|fd| !fd.revents().is_empty()).collect()
}

pub struct SocketRig {
    bench: Shared,
    worker: Worker<FakeOpener, VirtualWaiter>,
    changes: DeviceChanges,
    published: async_channel::Receiver<DeviceUpdate>,
    updates: Vec<(Millis, DeviceUpdate)>,
    turns: usize,
}

impl SocketRig {
    fn turn(&mut self) {
        self.turns += 1;
        assert!(
            self.turns < TURN_LIMIT,
            "the worker took more than {TURN_LIMIT} turns"
        );
        self.worker.turn();
        let now = self.bench.borrow().now;
        while let Ok(update) = self.published.try_recv() {
            self.updates.push((now, update));
        }
    }

    fn bench(&self) -> std::cell::RefMut<'_, Bench> {
        self.bench.borrow_mut()
    }
}

impl Rig for SocketRig {
    fn start(headset: FakeHeadset) -> Self {
        let bench = Rc::new(RefCell::new(Bench::new(headset)));
        let (changes, mailbox) = wake::mailbox().expect("a mailbox");
        let (outbox, published) = async_channel::unbounded();
        let root = bench.borrow().sysfs.root();
        let worker = Worker::new(Wiring {
            root,
            opener: FakeOpener(bench.clone()),
            waiter: VirtualWaiter(bench.clone()),
            mailbox,
            updates: outbox,
        });
        Self {
            bench,
            worker,
            changes,
            published,
            updates: Vec::new(),
            turns: 0,
        }
    }

    fn plug(&mut self) {
        self.bench().plug();
    }

    fn deny(&mut self) {
        self.bench().deny();
    }

    fn unplug(&mut self) {
        self.bench().unplug();
    }

    fn desire(&mut self, settings: HeadsetSettings, policy: SyncPolicy) {
        assert!(self.changes.send(Change::Desired { settings, policy }));
    }

    fn flap(&mut self, flap: Flap) {
        self.bench().flap(flap);
    }

    fn remove_command_node(&mut self) {
        self.bench().remove_command_node();
    }

    fn fail_next_write(&mut self) {
        self.bench().fail_next_write();
    }

    fn press(&mut self, change: SettingChange) {
        let mut bench = self.bench();
        let now = bench.now;
        let emissions = bench.headset.press(change, now);
        bench.emit(emissions);
    }

    fn link(&mut self, link: Link) {
        let mut bench = self.bench();
        let now = bench.now;
        let emissions = bench.headset.link_event(link, now);
        bench.emit(emissions);
    }

    fn battery(&mut self, battery: Percent) {
        let mut bench = self.bench();
        let now = bench.now;
        let emissions = bench.headset.battery_event(battery, now);
        bench.emit(emissions);
    }

    fn run_for(&mut self, duration: u64) {
        {
            let mut bench = self.bench();
            bench.horizon = bench.now.after(duration);
            bench.reached = false;
        }
        while !self.bench.borrow().reached {
            self.turn();
        }
    }

    fn now(&self) -> Millis {
        self.bench.borrow().now
    }

    fn phase(&self) -> Phase {
        self.worker.phase()
    }

    fn headset(&self) -> FakeHeadset {
        self.bench.borrow().headset.clone()
    }

    fn writes(&self) -> Vec<(Millis, Frame)> {
        self.bench.borrow().writes.clone()
    }

    fn updates(&self) -> Vec<(Millis, DeviceUpdate)> {
        self.updates.clone()
    }

    fn clear_logs(&mut self) {
        self.bench().writes.clear();
        self.updates.clear();
    }
}
