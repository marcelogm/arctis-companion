use super::echo::EchoFilter;
use super::exchange::{Outgoing, Unanswered};
use super::verify::{Progress, Transaction, same};
use crate::messages::device::{DeviceUpdate, SyncPolicy};
use crate::protocol::Reply;
use crate::protocol::status::Link;
use crate::settings::headset_settings::{HeadsetSettings, Setting, SettingChange};
use crate::time::Millis;

const WRITE_ORDER: [Setting; 7] = [
    Setting::MicVolume,
    Setting::Sidetone,
    Setting::MuteLight,
    Setting::VolumeLimiter,
    Setting::AutoPowerOff,
    Setting::Equalizer,
    Setting::WirelessMode,
];

#[derive(Clone, Debug, Default)]
pub(crate) struct Reconciler {
    desired: Option<HeadsetSettings>,
    observed: Option<HeadsetSettings>,
    failed: Vec<Setting>,
    transaction: Option<Transaction>,
    echoes: EchoFilter,
}

impl Reconciler {
    pub(crate) fn desire(
        &mut self,
        settings: HeadsetSettings,
        policy: SyncPolicy,
        now: Millis,
    ) -> Vec<DeviceUpdate> {
        self.failed.clear();
        match policy {
            SyncPolicy::Enforce => self.enforce(settings, now),
            SyncPolicy::Adopt => self.adopt(now),
        }
    }

    pub(crate) fn observe(&mut self, settings: HeadsetSettings) -> Vec<DeviceUpdate> {
        self.observed = Some(settings);
        self.observed_update()
    }

    pub(crate) fn clear_failures(&mut self) {
        self.failed.clear();
    }

    pub(crate) fn interrupt(&mut self, now: Millis) {
        self.transaction = None;
        self.echoes.close(now);
    }

    pub(crate) fn is_writing(&self) -> bool {
        self.transaction.is_some()
    }

    pub(crate) fn is_in_sync(&self) -> bool {
        !self.is_writing() && self.drift().is_empty()
    }

    pub(crate) fn due_at(&self, now: Millis) -> Option<Millis> {
        match &self.transaction {
            Some(transaction) => transaction.due_at(now),
            None => (!self.drift().is_empty()).then_some(now),
        }
    }

    pub(crate) fn next(&mut self, now: Millis) -> Option<Outgoing> {
        self.start_if_drifted();
        let outgoing = self.transaction.as_mut()?.next(now)?;
        if let Outgoing::Write(change) = &outgoing {
            self.echoes.record(change.clone());
        }
        Some(outgoing)
    }

    pub(crate) fn on_readback(&mut self, reply: &Reply, now: Millis) -> Vec<DeviceUpdate> {
        let readings = settings_in(reply);
        if let Some(observed) = &mut self.observed {
            readings
                .iter()
                .cloned()
                .for_each(|reading| observed.apply(reading));
        }
        let Some(transaction) = &mut self.transaction else {
            return Vec::new();
        };
        match transaction.check(&readings) {
            Progress::Pending => Vec::new(),
            Progress::Done => self.finish(Vec::new(), now),
            Progress::Failed(settings) => self.finish(settings, now),
        }
    }

    pub(crate) fn on_setting_event(
        &mut self,
        change: SettingChange,
        now: Millis,
    ) -> Vec<DeviceUpdate> {
        if self.echoes.is_echo(&change, now) {
            return Vec::new();
        }
        for settings in [&mut self.desired, &mut self.observed]
            .into_iter()
            .flatten()
        {
            settings.apply(change.clone());
        }
        let mut updates = self.drop_unwanted_targets(now);
        updates.push(DeviceUpdate::HeadsetChanged(change));
        updates
    }

    pub(crate) fn requeue_within_grace(&mut self, unanswered: Unanswered) -> bool {
        let Some(transaction) = &mut self.transaction else {
            return false;
        };
        let tolerated = transaction.tolerates(unanswered.asked_at);
        if tolerated {
            transaction.requeue(unanswered.query);
        }
        tolerated
    }

    pub(crate) fn link(&mut self, link: Link) {
        if let Some(transaction) = &mut self.transaction {
            transaction.link(link);
        }
    }

    pub(crate) fn link_lost_at(&self) -> Option<Millis> {
        self.transaction
            .as_ref()
            .and_then(Transaction::link_lost_at)
    }

    pub(crate) fn lost_link_for_good(&self, now: Millis) -> bool {
        self.transaction
            .as_ref()
            .is_some_and(|transaction| transaction.lost_link_for_good(now))
    }

    pub(crate) fn observed_update(&self) -> Vec<DeviceUpdate> {
        self.observed
            .clone()
            .map(DeviceUpdate::Observed)
            .into_iter()
            .collect()
    }

    fn enforce(&mut self, settings: HeadsetSettings, now: Millis) -> Vec<DeviceUpdate> {
        self.desired = Some(settings);
        self.drop_unwanted_targets(now)
    }

    fn adopt(&mut self, now: Millis) -> Vec<DeviceUpdate> {
        self.desired = None;
        self.interrupt(now);
        self.observed_update()
    }

    fn start_if_drifted(&mut self) {
        let changes = self.drift();
        if self.transaction.is_none() && !changes.is_empty() {
            self.transaction = Some(Transaction::new(changes));
        }
    }

    fn drop_unwanted_targets(&mut self, now: Millis) -> Vec<DeviceUpdate> {
        let (Some(transaction), Some(desired)) = (&mut self.transaction, &self.desired) else {
            return Vec::new();
        };
        match transaction.keep_wanted(desired) {
            Progress::Done => self.finish(Vec::new(), now),
            Progress::Pending | Progress::Failed(_) => Vec::new(),
        }
    }

    fn finish(&mut self, failed: Vec<Setting>, now: Millis) -> Vec<DeviceUpdate> {
        self.interrupt(now);
        let mut updates: Vec<_> = failed
            .iter()
            .map(|setting| DeviceUpdate::WriteFailed(*setting))
            .collect();
        self.failed.extend(failed);
        updates.extend(self.observed_update());
        updates
    }

    fn drift(&self) -> Vec<SettingChange> {
        match (&self.desired, &self.observed) {
            (Some(desired), Some(observed)) => drift(desired, observed, &self.failed),
            _ => Vec::new(),
        }
    }
}

fn drift(
    desired: &HeadsetSettings,
    observed: &HeadsetSettings,
    skipped: &[Setting],
) -> Vec<SettingChange> {
    WRITE_ORDER
        .into_iter()
        .filter(|setting| !skipped.contains(setting))
        .map(|setting| desired.value_of(setting))
        .filter(|wanted| !holds(observed, wanted))
        .collect()
}

fn holds(settings: &HeadsetSettings, wanted: &SettingChange) -> bool {
    same(&settings.value_of(wanted.setting()), wanted)
}

fn settings_in(reply: &Reply) -> Vec<SettingChange> {
    match reply {
        Reply::Audio(audio) => vec![
            SettingChange::MicVolume(audio.mic_volume),
            SettingChange::Sidetone(audio.sidetone),
            SettingChange::VolumeLimiter(audio.volume_limiter),
            SettingChange::Equalizer(audio.equalizer.clone()),
        ],
        Reply::Ux(ux) => vec![
            SettingChange::MuteLight(ux.mute_light),
            SettingChange::AutoPowerOff(ux.auto_power_off),
        ],
        Reply::Wireless(mode) => vec![SettingChange::WirelessMode(*mode)],
        Reply::Status(_) | Reply::Firmware(_) => Vec::new(),
    }
}
