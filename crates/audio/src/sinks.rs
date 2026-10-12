use arctis_core::messages::audio::{DefaultSink, SinkKind};
use arctis_core::protocol::model::{Model, ModelLookup, VENDOR_ID};

use crate::eq_names::EQ_SINK_NAME;
use crate::graph::{AUDIO_SINK, Graph, Node, ObjectId, Props};
use crate::keys::{DEVICE_API, DEVICE_PRODUCT_ID, DEVICE_VENDOR_ID, NODE_DESCRIPTION};
use crate::links::{self, Chain};

const BLUETOOTH_API: &str = "bluez5";
const NOVA_DESCRIPTION: &str = "Arctis Nova 5";
const EASY_EFFECTS_PREFIX: &str = "ee_";
const UNKNOWN: DefaultSink = DefaultSink {
    terminal: SinkKind::Other,
    via_equalizer: false,
    via_easy_effects: false,
};

#[must_use]
pub fn sink_kind(graph: &Graph, id: ObjectId) -> SinkKind {
    let Some(sink) = graph.node(id).filter(|node| node.is(AUDIO_SINK)) else {
        return SinkKind::Other;
    };
    if is_nova_bluetooth(sink) {
        SinkKind::NovaBluetooth
    } else if graph.device_of(sink).is_some_and(is_nova_dongle) {
        SinkKind::NovaDongle
    } else {
        SinkKind::Other
    }
}

fn is_nova_bluetooth(sink: &Node) -> bool {
    sink.props.get(DEVICE_API) == Some(BLUETOOTH_API)
        && sink
            .props
            .get(NODE_DESCRIPTION)
            .is_some_and(|description| description.starts_with(NOVA_DESCRIPTION))
}

fn is_nova_dongle(device: &Props) -> bool {
    let vendor = device.get(DEVICE_VENDOR_ID).and_then(parse_hex);
    let product = device.get(DEVICE_PRODUCT_ID).and_then(parse_hex);
    vendor == Some(VENDOR_ID)
        && product.is_some_and(|pid| matches!(Model::lookup(pid), ModelLookup::Supported(_)))
}

fn parse_hex(text: &str) -> Option<u16> {
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))?;
    u16::from_str_radix(digits, 16).ok()
}

#[must_use]
pub fn has_nova_bluetooth(graph: &Graph) -> bool {
    graph
        .nodes()
        .any(|(id, _)| sink_kind(graph, id) == SinkKind::NovaBluetooth)
}

#[must_use]
pub fn default_sink(graph: &Graph) -> Option<DefaultSink> {
    (!effective_default_missing(graph)).then(|| route(graph))
}

fn effective_default_missing(graph: &Graph) -> bool {
    graph
        .defaults()
        .audio_sink
        .as_deref()
        .is_some_and(|name| graph.node_named(name).is_none())
}

fn route(graph: &Graph) -> DefaultSink {
    default_node(graph)
        .and_then(|id| links::follow(graph, id))
        .map_or(UNKNOWN, |chain| describe(graph, &chain))
}

#[must_use]
pub fn default_node(graph: &Graph) -> Option<ObjectId> {
    graph.node_named(graph.defaults().current()?)
}

fn describe(graph: &Graph, chain: &Chain) -> DefaultSink {
    DefaultSink {
        terminal: sink_kind(graph, chain.terminal()),
        via_equalizer: passes_through_equalizer(graph, chain),
        via_easy_effects: chain_has(graph, chain, |name| name.starts_with(EASY_EFFECTS_PREFIX)),
    }
}

#[must_use]
pub fn passes_through_equalizer(graph: &Graph, chain: &Chain) -> bool {
    chain_has(graph, chain, |name| name == EQ_SINK_NAME)
}

fn chain_has(graph: &Graph, chain: &Chain, test: impl Fn(&str) -> bool) -> bool {
    chain
        .nodes()
        .iter()
        .filter_map(|id| graph.node(*id)?.name())
        .any(test)
}
