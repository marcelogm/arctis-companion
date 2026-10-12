use arctis_core::messages::audio::{AudioEvent, DefaultSink, SinkKind, StreamInfo};

use crate::eq_names::EQ_SINK_NAME;
use crate::graph::Graph;
use crate::sinks::{default_node, default_sink, has_nova_bluetooth};
use crate::streams::streams;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Settling {
    Insertion,
    Removal,
}

impl Settling {
    fn reached(self, sink: DefaultSink, graph: &Graph) -> bool {
        match self {
            Self::Insertion => sink.terminal == SinkKind::NovaBluetooth && sink.via_equalizer,
            Self::Removal => !sink.via_equalizer && restored_elsewhere(graph),
        }
    }
}

fn restored_elsewhere(graph: &Graph) -> bool {
    default_node(graph).is_some() && graph.defaults().current() != Some(EQ_SINK_NAME)
}

pub type Ticket = u64;

#[derive(Debug, Default)]
pub struct Published {
    default_sink: Option<DefaultSink>,
    streams: Option<Vec<StreamInfo>>,
    bluetooth_seen: bool,
    settling: Option<(Settling, Ticket)>,
    tickets: Ticket,
}

impl Published {
    pub fn settle(&mut self, settling: Settling) -> Ticket {
        self.tickets += 1;
        self.settling = Some((settling, self.tickets));
        self.tickets
    }

    pub fn stop_settling(&mut self, ticket: Ticket) {
        if self.settling.is_some_and(|(_, current)| current == ticket) {
            self.settling = None;
        }
    }

    pub fn changes(&mut self, graph: &Graph) -> Vec<AudioEvent> {
        [
            self.bluetooth_change(graph),
            self.default_sink_change(graph).map(AudioEvent::DefaultSink),
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

    fn default_sink_change(&mut self, graph: &Graph) -> Option<DefaultSink> {
        let current = default_sink(graph)?;
        if self.still_settling(current, graph) {
            return None;
        }
        replace_if_changed(&mut self.default_sink, current)
    }

    fn still_settling(&mut self, current: DefaultSink, graph: &Graph) -> bool {
        let Some((settling, _)) = self.settling else {
            return false;
        };
        let reached = settling.reached(current, graph);
        if reached {
            self.settling = None;
        }
        !reached
    }
}

fn replace_if_changed<T: Clone + PartialEq>(slot: &mut Option<T>, current: T) -> Option<T> {
    if slot.as_ref() == Some(&current) {
        return None;
    }
    *slot = Some(current.clone());
    Some(current)
}
