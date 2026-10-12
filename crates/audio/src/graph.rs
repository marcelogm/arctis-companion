use std::collections::BTreeMap;

use crate::keys::{
    CLIENT_ID, CONFIGURED_AUDIO_SINK, DEFAULT_AUDIO_SINK, DEVICE_ID, MEDIA_CLASS, NODE_NAME,
};
use crate::metadata_value::name_from_json;
use crate::volume::{Volume, VolumeChange};

pub type ObjectId = u32;

pub const AUDIO_SINK: &str = "Audio/Sink";
pub const OUTPUT_STREAM: &str = "Stream/Output/Audio";
pub const INPUT_STREAM: &str = "Stream/Input/Audio";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Props(BTreeMap<String, String>);

impl Props {
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    fn id(&self, key: &str) -> Option<ObjectId> {
        self.get(key)?.parse().ok()
    }
}

impl<'a> FromIterator<(&'a str, &'a str)> for Props {
    fn from_iter<I: IntoIterator<Item = (&'a str, &'a str)>>(pairs: I) -> Self {
        Self(
            pairs
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect(),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Node {
    pub props: Props,
    pub running: bool,
    pub volume: Volume,
}

impl Node {
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.props.get(NODE_NAME)
    }

    #[must_use]
    pub fn is(&self, class: &str) -> bool {
        self.props.get(MEDIA_CLASS) == Some(class)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Link {
    pub output: ObjectId,
    pub input: ObjectId,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Defaults {
    pub audio_sink: Option<String>,
    pub configured_audio_sink: Option<String>,
}

impl Defaults {
    #[must_use]
    pub fn current(&self) -> Option<&str> {
        self.audio_sink
            .as_deref()
            .or(self.configured_audio_sink.as_deref())
    }

    #[must_use]
    pub fn configured(&self) -> Option<&str> {
        self.configured_audio_sink.as_deref()
    }

    #[must_use]
    pub fn mentions(&self, name: &str) -> bool {
        [self.audio_sink.as_deref(), self.configured()].contains(&Some(name))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Graph {
    nodes: BTreeMap<ObjectId, Node>,
    devices: BTreeMap<ObjectId, Props>,
    clients: BTreeMap<ObjectId, Props>,
    links: BTreeMap<ObjectId, Link>,
    defaults: Defaults,
    defaults_source: Option<ObjectId>,
}

impl Graph {
    pub fn put_node(&mut self, id: ObjectId, props: Props) {
        self.nodes.entry(id).or_default().props = props;
    }

    pub fn set_running(&mut self, id: ObjectId, running: bool) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.running = running;
        }
    }

    pub fn change_volume(&mut self, id: ObjectId, change: &VolumeChange) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.volume = node.volume.merged(change);
        }
    }

    #[must_use]
    pub fn volume_of(&self, name: &str) -> Option<Volume> {
        let volume = &self.node(self.node_named(name)?)?.volume;
        (!volume.channels.is_empty()).then(|| volume.clone())
    }

    pub fn put_device(&mut self, id: ObjectId, props: Props) {
        self.devices.insert(id, props);
    }

    pub fn put_client(&mut self, id: ObjectId, props: Props) {
        self.clients.insert(id, props);
    }

    pub fn put_link(&mut self, id: ObjectId, link: Link) {
        self.links.insert(id, link);
    }

    pub fn remove(&mut self, id: ObjectId) {
        self.nodes.remove(&id);
        self.devices.remove(&id);
        self.clients.remove(&id);
        self.links.remove(&id);
        if self.defaults_source == Some(id) {
            self.defaults_source = None;
            self.clear_defaults();
        }
    }

    pub fn take_defaults_from(&mut self, metadata: ObjectId) {
        self.defaults_source = Some(metadata);
    }

    pub fn set_default(&mut self, key: &str, value: Option<&str>) {
        let name = value.and_then(name_from_json);
        match key {
            DEFAULT_AUDIO_SINK => self.defaults.audio_sink = name,
            CONFIGURED_AUDIO_SINK => self.defaults.configured_audio_sink = name,
            _ => {}
        }
    }

    pub fn clear_defaults(&mut self) {
        self.defaults = Defaults::default();
    }

    #[must_use]
    pub fn defaults(&self) -> &Defaults {
        &self.defaults
    }

    #[must_use]
    pub fn node(&self, id: ObjectId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = (ObjectId, &Node)> {
        self.nodes.iter().map(|(id, node)| (*id, node))
    }

    #[must_use]
    pub fn node_named(&self, name: &str) -> Option<ObjectId> {
        self.nodes()
            .find(|(_, node)| node.name() == Some(name))
            .map(|(id, _)| id)
    }

    #[must_use]
    pub fn device_of(&self, node: &Node) -> Option<&Props> {
        self.devices.get(&node.props.id(DEVICE_ID)?)
    }

    #[must_use]
    pub fn client_of(&self, node: &Node) -> Option<&Props> {
        self.clients.get(&node.props.id(CLIENT_ID)?)
    }

    pub fn inputs_fed_by(&self, id: ObjectId) -> impl Iterator<Item = ObjectId> {
        self.links
            .values()
            .filter(move |link| link.output == id)
            .map(|link| link.input)
    }
}
