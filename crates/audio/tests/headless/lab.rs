use std::thread;
use std::time::{Duration, Instant};

use arctis_audio::AudioHandle;
use arctis_core::eq::{Curve, library};
use arctis_core::messages::audio::{AudioCommand, AudioEvent, DefaultSink, SinkKind};

use crate::tools;

pub const HEADLESS: &str = "needs the private PipeWire of crates/audio/tests/headless/run.sh";
pub const BLUEZ: &str = "bluez_output.AA_BB_CC_DD_EE_FF.1";
pub const USB: &str = "alsa_output.usb-SteelSeries_SteelSeries_Arctis_Nova_5-00.iec958-stereo";
pub const EQ_SINK: &str = "arctis_companion_eq";
const FULL_VOLUME: &str = "{ channelVolumes = [ 1.0, 1.0 ], mute = false }";
const PATIENCE: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(20);

pub fn bluetooth(via_equalizer: bool) -> AudioEvent {
    AudioEvent::DefaultSink(DefaultSink {
        terminal: SinkKind::NovaBluetooth,
        via_equalizer,
        via_easy_effects: false,
    })
}

pub fn flat() -> Curve {
    library::flat().curve().clone()
}

pub fn eventually(what: &str, test: impl Fn() -> bool) {
    let deadline = Instant::now() + PATIENCE;
    while !test() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        thread::sleep(POLL);
    }
}

pub struct Lab {
    pub audio: AudioHandle,
}

impl Lab {
    pub fn open() -> Self {
        assert_eq!(
            std::env::var("ARCTIS_HEADLESS_PIPEWIRE").as_deref(),
            Ok("1"),
            "{HEADLESS}"
        );
        tools::set_configured_default(BLUEZ);
        tools::set_props(BLUEZ, FULL_VOLUME);
        eventually("the bluez default", || {
            tools::configured_default().as_deref() == Some(BLUEZ)
        });
        let lab = Self {
            audio: AudioHandle::spawn().expect("audio thread"),
        };
        lab.wait_for("Available", |event| *event == AudioEvent::Available);
        lab
    }

    pub fn send(&self, command: AudioCommand) {
        assert!(self.audio.commands.send(command), "audio thread gone");
    }

    pub fn wait_for(&self, what: &str, test: impl Fn(&AudioEvent) -> bool) -> AudioEvent {
        let deadline = Instant::now() + PATIENCE;
        std::iter::repeat_with(|| self.next_event(deadline, what))
            .flatten()
            .find(|event| test(event))
            .expect("an endless iterator")
    }

    fn next_event(&self, deadline: Instant, what: &str) -> Option<AudioEvent> {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        let event = self.audio.events.try_recv().ok();
        if event.is_none() {
            thread::sleep(POLL);
        }
        event
    }

    pub fn insert(&self, curve: Curve) {
        self.send(AudioCommand::InsertEqualizer(curve));
        self.wait_for("the equalizer as default", |event| {
            *event == bluetooth(true)
        });
    }
}
