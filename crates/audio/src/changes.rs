use arctis_core::messages::audio::{AudioEvent, DefaultSink, StreamInfo};

use crate::graph::Graph;
use crate::outputs::{default_sink, has_nova_bluetooth};
use crate::streams::streams;

#[derive(Debug, Default)]
pub struct Published {
    default_sink: Option<DefaultSink>,
    streams: Option<Vec<StreamInfo>>,
    bluetooth_seen: bool,
}

impl Published {
    pub fn changes(&mut self, graph: &Graph) -> Vec<AudioEvent> {
        [
            self.bluetooth_change(graph),
            replace_if_changed(&mut self.default_sink, default_sink(graph))
                .map(AudioEvent::DefaultSink),
            replace_if_changed(&mut self.streams, streams(graph)).map(AudioEvent::Streams),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn bluetooth_change(&mut self, graph: &Graph) -> Option<AudioEvent> {
        if self.bluetooth_seen || !has_nova_bluetooth(graph) {
            return None;
        }
        self.bluetooth_seen = true;
        Some(AudioEvent::BluetoothNodeSeen)
    }
}

fn replace_if_changed<T: Clone + PartialEq>(slot: &mut Option<T>, current: T) -> Option<T> {
    if slot.as_ref() == Some(&current) {
        return None;
    }
    *slot = Some(current.clone());
    Some(current)
}
