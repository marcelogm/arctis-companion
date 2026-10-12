use std::process::{Child, Command};

use arctis_core::messages::audio::{AudioEvent, SoundPath, StreamInfo};

use crate::lab::{EQ_SINK_NAME, Lab, eventually, flat};
use crate::tone::Tone;
use crate::tools;

fn on_bluetooth<'a>(event: &'a AudioEvent, id: &str) -> Option<&'a StreamInfo> {
    let AudioEvent::Streams(streams) = event else {
        return None;
    };
    streams.iter().find(|stream| {
        stream.path == Some(SoundPath::Bluetooth)
            && stream.running
            && stream.ids.as_slice().iter().any(|known| known == id)
    })
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn identifiers_come_from_node_and_client_props() {
    let lab = Lab::open();
    lab.insert(flat());
    let _pulse_style = Tone::play(
        vec![
            ("application.id", "org.example.NodePlayer"),
            ("application.process.binary", "NodePlayer"),
        ],
        Vec::new(),
    );
    let _native_style = Tone::play(
        vec![("application.name", "Client Player")],
        vec![("pipewire.access.portal.app_id", "org.example.ClientPlayer")],
    );
    let found = lab.wait_for("both players", |event| {
        on_bluetooth(event, "nodeplayer").is_some()
            && on_bluetooth(event, "org.example.ClientPlayer").is_some()
    });
    let pulse = on_bluetooth(&found, "nodeplayer").expect("node player");
    assert_eq!(
        pulse.ids.as_slice(),
        ["org.example.NodePlayer", "nodeplayer"]
    );
    let native = on_bluetooth(&found, "org.example.ClientPlayer").expect("client player");
    assert_eq!(native.name, "Client Player");
}

struct Loopback(Child);

impl Drop for Loopback {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn intermediate_filter() -> Loopback {
    let playback = format!(
        "{{ node.name = chain_filter_out target.object = {EQ_SINK_NAME} node.dont-reconnect = true audio.position = [ FL FR ] }}"
    );
    let child = Command::new("pw-loopback")
        .args([
            "--capture-props",
            "{ media.class = Audio/Sink node.name = chain_filter audio.position = [ FL FR ] }",
            "--playback-props",
            &playback,
        ])
        .spawn()
        .expect("pw-loopback");
    eventually("the intermediate filter", || {
        tools::node_id("chain_filter").is_some()
    });
    Loopback(child)
}

#[test]
#[ignore = "needs the private PipeWire of crates/audio/tests/headless/run.sh"]
fn a_stream_behind_an_intermediate_node_is_on_the_bluetooth_path() {
    let lab = Lab::open();
    lab.insert(flat());
    let _filter = intermediate_filter();
    let _player = Tone::play(
        vec![
            ("application.id", "org.example.Chained"),
            ("target.object", "chain_filter"),
        ],
        Vec::new(),
    );
    lab.wait_for("the chained player", |event| {
        on_bluetooth(event, "org.example.Chained").is_some()
    });
    assert!(tools::node_id(EQ_SINK_NAME).is_some());
}
