use super::echo::EchoFilter;
use super::exchange::{Outgoing, Unanswered};
use super::grace::Grace;
use super::verify::{Progress, Transaction, same};
use crate::messages::device::{DeviceUpdate, SyncPolicy};
use crate::protocol::reply::{AudioReadback, UxReadback};
use crate::protocol::status::Link;
use crate::protocol::{Query, Reply};
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
    unknown: Vec<Setting>,
    transaction: Option<Transaction>,
    echoes: EchoFilter,
    grace: Grace,
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

    pub(crate) fn start_syncing(&mut self) {
        self.failed.clear();
        self.unknown.clear();
    }

    pub(crate) fn assume(&mut self, query: Query) -> Option<Reply> {
        let reply = assumed(query, self.desired.as_ref()?)?;
        self.unknown
            .extend(settings_in(&reply).iter().map(SettingChange::setting));
        Some(reply)
    }

    pub(crate) fn interrupt(&mut self, now: Millis) {
        self.transaction = None;
        self.echoes.close(now);
    }

    pub(crate) fn is_writing(&self) -> bool {
        self.transaction.is_some()
    }

    pub(crate) fn waits_for_link(&self, now: Millis) -> bool {
        self.is_writing() || self.grace.covers(now)
    }

    pub(crate) fn end_grace(&mut self) {
        self.grace.close();
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
            self.wrote(change, now);
        }
        Some(outgoing)
    }

    pub(crate) fn on_readback(&mut self, reply: &Reply, now: Millis) -> Vec<DeviceUpdate> {
        let readings = settings_in(reply);
        self.unknown
            .retain(|setting| !readings.iter().any(|reading| reading.setting() == *setting));
        if let Some(observed) = &mut self.observed {
            readings
                .iter()
                .cloned()
                .for_each(|reading| observed.apply(reading));
        }
        self.verify(&readings, now)
    }

    pub(crate) fn on_unreadable(&mut self, now: Millis) -> Vec<DeviceUpdate> {
        self.verify(&[], now)
    }

    fn verify(&mut self, readings: &[SettingChange], now: Millis) -> Vec<DeviceUpdate> {
        let Some(transaction) = &mut self.transaction else {
            return Vec::new();
        };
        match transaction.check(readings) {
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
        self.unknown.retain(|setting| *setting != change.setting());
        updates.push(DeviceUpdate::HeadsetChanged(change));
        updates
    }

    pub(crate) fn requeue_within_grace(&mut self, unanswered: Unanswered) -> bool {
        let Some(transaction) = &mut self.transaction else {
            return false;
        };
        let tolerated = self.grace.covers(unanswered.asked_at);
        if tolerated {
            transaction.requeue(unanswered.query);
        }
        tolerated
    }

    pub(crate) fn link(&mut self, link: Link) {
        self.grace.link(link);
        if let Some(transaction) = &mut self.transaction {
            transaction.link(link);
        }
    }

    pub(crate) fn link_lost_at(&self) -> Option<Millis> {
        let written = self
            .transaction
            .as_ref()
            .and_then(Transaction::link_lost_at);
        written.max(self.grace.link_lost_at())
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

    fn wrote(&mut self, change: &SettingChange, now: Millis) {
        if change.setting() == Setting::WirelessMode {
            self.grace.open(now);
        }
        self.echoes.record(change.clone());
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
        let overwritten = transaction.keep_wanted(desired);
        let done = transaction.is_done();
        self.unknown.extend(overwritten);
        if done {
            self.finish(Vec::new(), now)
        } else {
            Vec::new()
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
        let (Some(desired), Some(observed)) = (&self.desired, &self.observed) else {
            return Vec::new();
        };
        WRITE_ORDER
            .into_iter()
            .filter(|setting| !self.failed.contains(setting))
            .map(|setting| desired.value_of(setting))
            .filter(|wanted| self.unknown.contains(&wanted.setting()) || !holds(observed, wanted))
            .collect()
    }
}

fn holds(settings: &HeadsetSettings, wanted: &SettingChange) -> bool {
    same(&settings.value_of(wanted.setting()), wanted)
}

fn assumed(query: Query, desired: &HeadsetSettings) -> Option<Reply> {
    match query {
        Query::Audio => Some(Reply::Audio(AudioReadback {
            equalizer: desired.equalizer.clone(),
            mic_volume: desired.mic_volume,
            sidetone: desired.sidetone,
            volume_limiter: desired.volume_limiter,
        })),
        Query::Ux => Some(Reply::Ux(UxReadback {
            auto_power_off: desired.auto_power_off,
            mute_light: desired.mute_light,
        })),
        Query::Wireless => Some(Reply::Wireless(desired.wireless_mode)),
        Query::Status | Query::Firmware => None,
    }
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
