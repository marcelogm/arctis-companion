use super::timing::ECHO_WINDOW;
use crate::settings::headset_settings::SettingChange;
use crate::time::Millis;

#[derive(Clone, Debug)]
struct Written {
    change: SettingChange,
    until: Option<Millis>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EchoFilter {
    written: Vec<Written>,
}

impl EchoFilter {
    pub(crate) fn record(&mut self, change: SettingChange) {
        self.written.push(Written {
            change,
            until: None,
        });
    }

    pub(crate) fn close(&mut self, now: Millis) {
        self.forget_expired(now);
        let until = now.after(ECHO_WINDOW);
        for written in self
            .written
            .iter_mut()
            .filter(|written| written.until.is_none())
        {
            written.until = Some(until);
        }
    }

    pub(crate) fn is_echo(&mut self, change: &SettingChange, now: Millis) -> bool {
        self.forget_expired(now);
        self.written.iter().any(|written| written.change == *change)
    }

    fn forget_expired(&mut self, now: Millis) {
        self.written
            .retain(|written| written.until.is_none_or(|until| now <= until));
    }
}
