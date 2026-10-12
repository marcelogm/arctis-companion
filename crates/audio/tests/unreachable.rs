use std::time::{Duration, Instant};

use arctis_audio::AudioHandle;
use arctis_core::messages::audio::AudioEvent;

const PATIENCE: Duration = Duration::from_secs(5);

#[test]
#[ignore = "run by crates/audio/tests/headless/session.sh before PipeWire starts"]
fn no_pipewire_reports_unavailable_and_still_quits() {
    assert_eq!(
        std::env::var("ARCTIS_HEADLESS_PIPEWIRE").as_deref(),
        Ok("1"),
        "run through crates/audio/tests/headless/run.sh"
    );
    let audio = AudioHandle::spawn().expect("audio thread");
    let first = audio
        .events
        .recv_blocking()
        .expect("an event before the thread ends");
    assert_eq!(first, AudioEvent::Unavailable);
    let started = Instant::now();
    drop(audio);
    assert!(started.elapsed() < PATIENCE);
}
