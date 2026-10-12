use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use arctis_audio::AudioHandle;
use arctis_core::messages::audio::AudioEvent;

const PATIENCE: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(20);

struct Daemon(Child);

impl Daemon {
    fn start() -> Self {
        let child = Command::new("pipewire")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("pipewire");
        Self(child)
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = Command::new("kill").arg(self.0.id().to_string()).status();
        let _ = self.0.wait();
    }
}

fn wait_for(audio: &AudioHandle, expected: &AudioEvent) {
    let deadline = Instant::now() + PATIENCE;
    while audio.events.try_recv().ok().as_ref() != Some(expected) {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {expected:?}"
        );
        thread::sleep(POLL);
    }
}

#[test]
#[ignore = "run by crates/audio/tests/headless/session.sh before PipeWire starts"]
fn pipewire_coming_and_going_is_announced() {
    assert_eq!(
        std::env::var("ARCTIS_HEADLESS_PIPEWIRE").as_deref(),
        Ok("1"),
        "run through crates/audio/tests/headless/run.sh"
    );
    let audio = AudioHandle::spawn().expect("audio thread");
    wait_for(&audio, &AudioEvent::Unavailable);
    let daemon = Daemon::start();
    wait_for(&audio, &AudioEvent::Available);
    drop(daemon);
    wait_for(&audio, &AudioEvent::Unavailable);
    let started = Instant::now();
    drop(audio);
    assert!(started.elapsed() < PATIENCE);
}
