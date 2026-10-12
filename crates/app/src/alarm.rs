use std::cell::RefCell;
use std::time::Duration;

use arctis_core::time::Millis;
use gtk::glib;

use crate::ports::{Alarm, Clock};

const MICROS_PER_MILLI: i64 = 1_000;

#[derive(Default)]
pub struct GlibAlarm {
    pending: RefCell<Option<glib::JoinHandle<()>>>,
}

impl Alarm for GlibAlarm {
    fn schedule(&self, delay: Duration, wake: Box<dyn FnOnce()>) {
        let job = glib::spawn_future_local(async move {
            glib::timeout_future(delay).await;
            wake();
        });
        if let Some(previous) = self.pending.replace(Some(job)) {
            previous.abort();
        }
    }
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Millis {
        Millis(u64::try_from(glib::real_time() / MICROS_PER_MILLI).unwrap_or_default())
    }
}
