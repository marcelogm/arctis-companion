use arctis_core::messages::audio::{DefaultSink, SinkKind};
use arctis_core::protocol::model::{Model, ModelLookup, VENDOR_ID};

use crate::graph::{AUDIO_SINK, Graph, Node, ObjectId, Props};
use crate::links::{self, Chain};

pub const EQ_SINK_NAME: &str = "arctis_companion_eq";
pub const EQ_OUTPUT_NAME: &str = "arctis_companion_eq_out";
pub const EQ_DESCRIPTION: &str = "Arctis Nova 5 (Equalizer)";
pub const EQ_LINK_GROUP: &str = "arctis-companion-eq";
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
    sink.props.get("device.api") == Some(BLUETOOTH_API)
        && sink
            .props
            .get("node.description")
            .is_some_and(|description| description.starts_with(NOVA_DESCRIPTION))
}

fn is_nova_dongle(device: &Props) -> bool {
    let vendor = device.get("device.vendor.id").and_then(parse_hex);
    let product = device.get("device.product.id").and_then(parse_hex);
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
pub fn default_sink(graph: &Graph) -> DefaultSink {
    graph
        .defaults()
        .current()
        .and_then(|name| graph.node_named(name))
        .and_then(|id| links::follow(graph, id))
        .map_or(UNKNOWN, |chain| describe(graph, &chain))
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

#[must_use]
pub fn insertion_target(graph: &Graph) -> Option<String> {
    let name = graph.defaults().current()?;
    let id = graph.node_named(name)?;
    (sink_kind(graph, id) == SinkKind::NovaBluetooth).then(|| name.to_owned())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InsertStep {
    Create { replaced: String },
    Retarget,
    Skip,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemoveStep {
    Restore(String),
    LeaveDefault,
    NothingInserted,
}

#[derive(Debug, Default)]
pub struct Outputs {
    replaced: Option<String>,
}

impl Outputs {
    pub fn insert(&mut self, graph: &Graph) -> InsertStep {
        if self.replaced.is_some() {
            return InsertStep::Retarget;
        }
        let Some(replaced) = insertion_target(graph) else {
            return InsertStep::Skip;
        };
        self.replaced = Some(replaced.clone());
        InsertStep::Create { replaced }
    }

    pub fn remove(&mut self, graph: &Graph) -> RemoveStep {
        match self.replaced.take() {
            None => RemoveStep::NothingInserted,
            Some(replaced) if graph.defaults().mentions(EQ_SINK_NAME) => {
                RemoveStep::Restore(replaced)
            }
            Some(_) => RemoveStep::LeaveDefault,
        }
    }

    #[must_use]
    pub fn manager_denied(&self, graph: &Graph) -> bool {
        let defaults = graph.defaults();
        self.replaced.as_deref().is_some_and(|replaced| {
            defaults.configured() != Some(EQ_SINK_NAME) && defaults.current() == Some(replaced)
        })
    }

    #[must_use]
    pub fn is_inserted(&self) -> bool {
        self.replaced.is_some()
    }
}
