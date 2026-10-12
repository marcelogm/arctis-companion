use std::io;
use std::thread::{self, JoinHandle};

use arctis_core::messages::device::DeviceUpdate;

use crate::discovery::SysfsRoot;
use crate::port::HidrawOpener;
use crate::waiter::PollWaiter;
use crate::wake::{self, DeviceChanges};
use crate::worker::{Wiring, Worker};

const THREAD_NAME: &str = "device";

pub struct DeviceHandle {
    pub changes: DeviceChanges,
    pub updates: async_channel::Receiver<DeviceUpdate>,
    thread: Option<JoinHandle<()>>,
}

impl DeviceHandle {
    pub fn spawn(root: SysfsRoot) -> io::Result<Self> {
        let (changes, mailbox) = wake::mailbox()?;
        let (publisher, updates) = async_channel::unbounded();
        let wiring = Wiring {
            root,
            opener: HidrawOpener,
            waiter: PollWaiter::new(),
            mailbox,
            updates: publisher,
        };
        let thread = thread::Builder::new()
            .name(THREAD_NAME.to_owned())
            .spawn(move || Worker::new(wiring).run())?;
        Ok(Self {
            changes,
            updates,
            thread: Some(thread),
        })
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        self.changes.quit();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
