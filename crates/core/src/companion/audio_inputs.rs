use super::action::Action;
use super::app_settings::PersistedChange;
use super::quitting::Quit;
use super::reducer::Companion;
use super::routing::{Route, needs_equalizer};
use super::seen_apps::{newly_playing, remember};
use crate::eq::curve::Curve;
use crate::messages::audio::{AudioCommand, AudioEvent, SoundPath, StreamInfo};

impl Companion {
    pub(super) fn audio_event(&mut self, event: &AudioEvent, out: &mut Vec<Action>) {
        if let AudioEvent::Streams(streams) = event {
            self.record_seen_apps(streams, out);
        }
        let route = self.audio.route();
        self.audio.apply(event);
        if *event == AudioEvent::BluetoothNodeSeen || self.audio.route() == Route::Bluetooth {
            self.bluetooth_seen(out);
        }
        if self.audio.route() != route {
            self.selection.end_manual_picks();
        }
        self.place_equalizer(out);
    }

    pub(super) fn remove_equalizer(&mut self, out: &mut Vec<Action>) {
        if self.audio.equalizer.take().is_some() {
            out.push(Action::SendAudio(AudioCommand::RemoveEqualizer));
        }
    }

    pub(super) fn retarget_system_curve(&mut self, curve: Curve, out: &mut Vec<Action>) -> bool {
        let running = self.audio.equalizer.as_ref();
        if running.is_none_or(|current| *current == curve) {
            return false;
        }
        self.audio.equalizer = Some(curve.clone());
        out.push(Action::SendAudio(AudioCommand::Retarget(curve)));
        true
    }

    fn record_seen_apps(&mut self, streams: &[StreamInfo], out: &mut Vec<Action>) {
        for app in newly_playing(&self.audio.streams, streams) {
            self.seen_apps = remember(&self.seen_apps, app.clone());
            out.push(Action::RecordSeenApp(app));
        }
    }

    fn bluetooth_seen(&mut self, out: &mut Vec<Action>) {
        if !self.settings.bluetooth_sink_seen {
            self.settings.bluetooth_sink_seen = true;
            out.push(Action::Persist(PersistedChange::BluetoothSinkSeen));
        }
    }

    fn place_equalizer(&mut self, out: &mut Vec<Action>) {
        let wanted =
            self.audio.default_sink.is_some_and(needs_equalizer) && self.quit == Quit::Running;
        if wanted && self.audio.equalizer.is_none() {
            let curve = self.active_curve(SoundPath::Bluetooth);
            self.audio.equalizer = Some(curve.clone());
            out.push(Action::SendAudio(AudioCommand::InsertEqualizer(curve)));
        } else if self.audio.route() != Route::Bluetooth || self.audio.easy_effects_active() {
            self.remove_equalizer(out);
        }
    }
}
