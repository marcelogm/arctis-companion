use super::action::Action;
use super::adoption::AdoptionStatus;
use super::reducer::Companion;
use super::timing::QUIT_GRACE;
use crate::messages::audio::SoundPath;
use crate::settings::headset_settings::Setting;
use crate::time::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Quit {
    Running,
    Waiting(Millis),
    Done,
}

impl Quit {
    pub(super) const fn deadline(self) -> Option<Millis> {
        match self {
            Self::Waiting(at) => Some(at),
            Self::Running | Self::Done => None,
        }
    }
}

impl Companion {
    pub(super) fn start_quit(&mut self, now: Millis, out: &mut Vec<Action>) {
        if self.quit == Quit::Running {
            self.quit = Quit::Waiting(now.after(QUIT_GRACE));
            self.remove_equalizer(out);
            out.extend(self.battery.disconnect(now).map(Action::RecordBattery));
        }
    }

    pub(super) fn finish_quit(&mut self, now: Millis, out: &mut Vec<Action>) {
        let Quit::Waiting(deadline) = self.quit else {
            return;
        };
        if now >= deadline || !self.awaiting_route_curve() {
            self.quit = Quit::Done;
            out.push(Action::ReadyToQuit);
        }
    }

    fn awaiting_route_curve(&self) -> bool {
        let verified = self.on_headset(&self.active_curve(SoundPath::Dongle));
        let failed = self.dongle.not_applied.contains(&Setting::Equalizer);
        self.dongle.is_ready() && self.adoption() == AdoptionStatus::Adopted && !verified && !failed
    }
}
