use arctis_core::messages::audio::{AppIdentifiers, SinkKind, SoundPath, StreamInfo};

use crate::graph::{Graph, Node, OUTPUT_STREAM, ObjectId, Props};
use crate::links::{self, LINK_GROUP};
use crate::outputs::{EQ_OUTPUT_NAME, passes_through_equalizer, sink_kind};

const PORTAL_APP_ID: &str = "pipewire.access.portal.app_id";
const APPLICATION_ID: &str = "application.id";
const PROCESS_BINARY: &str = "application.process.binary";
const APPLICATION_NAME: &str = "application.name";

#[must_use]
pub fn streams(graph: &Graph) -> Vec<StreamInfo> {
    graph
        .nodes()
        .filter(|(_, node)| is_app_stream(node))
        .filter_map(|(id, node)| stream_info(graph, id, node))
        .collect()
}

fn is_app_stream(node: &Node) -> bool {
    node.is(OUTPUT_STREAM)
        && node.props.get(LINK_GROUP).is_none()
        && node.name() != Some(EQ_OUTPUT_NAME)
}

fn stream_info(graph: &Graph, id: ObjectId, node: &Node) -> Option<StreamInfo> {
    let client = graph.client_of(node);
    let ids = identifiers(&node.props, client);
    let name = prop(&node.props, client, APPLICATION_NAME)
        .or(ids.key())?
        .to_owned();
    Some(StreamInfo {
        ids,
        name,
        running: node.running,
        path: path_of(graph, id),
    })
}

#[must_use]
pub fn identifiers(node: &Props, client: Option<&Props>) -> AppIdentifiers {
    AppIdentifiers::new(
        prop(node, client, PORTAL_APP_ID),
        prop(node, client, APPLICATION_ID),
        prop(node, client, PROCESS_BINARY),
    )
}

fn prop<'a>(node: &'a Props, client: Option<&'a Props>, key: &str) -> Option<&'a str> {
    node.get(key).or_else(|| client?.get(key))
}

fn path_of(graph: &Graph, id: ObjectId) -> Option<SoundPath> {
    let chain = links::follow(graph, id)?;
    if passes_through_equalizer(graph, &chain) {
        return Some(SoundPath::Bluetooth);
    }
    (sink_kind(graph, chain.terminal()) == SinkKind::NovaDongle).then_some(SoundPath::Dongle)
}
