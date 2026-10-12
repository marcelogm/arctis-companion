use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use pipewire as pw;
use pw::client::{Client, ClientChangeMask, ClientInfoRef};
use pw::device::{Device, DeviceChangeMask, DeviceInfoRef};
use pw::metadata::Metadata;
use pw::node::{Node, NodeChangeMask, NodeInfoRef, NodeState};
use pw::proxy::{Listener, ProxyT};
use pw::registry::{GlobalObject, RegistryRc, RegistryWeak};
use pw::spa::param::ParamType;
use pw::spa::pod::Pod;
use pw::spa::utils::dict::DictRef;
use pw::types::ObjectType;

use crate::graph::{CONFIGURED_AUDIO_SINK, Graph, Link, ObjectId, Props, json_for_name};
use crate::volume::volume_change;

const METADATA_NAME: &str = "metadata.name";
const DEFAULT_METADATA: &str = "default";
const JSON_TYPE: &str = "Spa:String:JSON";
const LINK_OUTPUT_NODE: &str = "link.output.node";
const LINK_INPUT_NODE: &str = "link.input.node";

pub type Changed = Rc<dyn Fn()>;

pub struct Registry {
    graph: Rc<RefCell<Graph>>,
    proxies: Rc<RefCell<Proxies>>,
    _listener: pw::registry::Listener,
    _proxy: RegistryRc,
}

#[derive(Default)]
struct Proxies {
    nodes: HashMap<ObjectId, Node>,
    metadata: Option<Metadata>,
    others: HashMap<ObjectId, Box<dyn ProxyT>>,
    listeners: HashMap<ObjectId, Box<dyn Listener>>,
}

impl Proxies {
    fn forget(&mut self, id: ObjectId) {
        self.listeners.remove(&id);
        self.nodes.remove(&id);
        self.others.remove(&id);
    }
}

impl Registry {
    #[must_use]
    pub fn watch(registry: RegistryRc, changed: Changed) -> Self {
        let watcher = Watcher {
            graph: Rc::new(RefCell::new(Graph::default())),
            changed,
        };
        let proxies = Rc::new(RefCell::new(Proxies::default()));
        let binder = Binder {
            registry: registry.downgrade(),
            proxies: proxies.clone(),
            watcher: watcher.clone(),
        };
        let remover = Remover {
            proxies: proxies.clone(),
            watcher: watcher.clone(),
        };
        let listener = registry
            .add_listener_local()
            .global(move |global| binder.bind(global))
            .global_remove(move |id| remover.forget(id))
            .register();
        Self {
            graph: watcher.graph,
            proxies,
            _listener: listener,
            _proxy: registry,
        }
    }

    #[must_use]
    pub fn graph(&self) -> Ref<'_, Graph> {
        self.graph.borrow()
    }

    pub fn write_default_sink(&self, name: &str) {
        if let Some(metadata) = &self.proxies.borrow().metadata {
            metadata.set_property(
                0,
                CONFIGURED_AUDIO_SINK,
                Some(JSON_TYPE),
                Some(&json_for_name(name)),
            );
        }
    }

    pub fn set_node_props(&self, name: &str, props: &Pod) {
        let Some(id) = self.graph.borrow().node_named(name) else {
            return;
        };
        if let Some(node) = self.proxies.borrow().nodes.get(&id) {
            node.set_param(ParamType::Props, 0, props);
        }
    }
}

#[derive(Clone)]
struct Watcher {
    graph: Rc<RefCell<Graph>>,
    changed: Changed,
}

impl Watcher {
    fn update(&self, edit: impl FnOnce(&mut Graph)) {
        edit(&mut self.graph.borrow_mut());
        (self.changed)();
    }

    fn node_info(&self, id: ObjectId, info: &NodeInfoRef) {
        let changed = info.change_mask().contains(NodeChangeMask::PROPS);
        let props = changed_props(changed, info.props());
        let running = matches!(info.state(), NodeState::Running);
        self.update(|graph| refresh_node(graph, id, props, running));
    }

    fn node_param(&self, id: ObjectId, kind: ParamType, param: Option<&Pod>) {
        let change = param
            .filter(|_| kind == ParamType::Props)
            .map(volume_change)
            .unwrap_or_default();
        if !change.is_empty() {
            self.update(|graph| graph.change_volume(id, &change));
        }
    }

    fn device_info(&self, id: ObjectId, info: &DeviceInfoRef) {
        let changed = info.change_mask().contains(DeviceChangeMask::PROPS);
        if let Some(props) = changed_props(changed, info.props()) {
            self.update(|graph| graph.put_device(id, props));
        }
    }

    fn client_info(&self, id: ObjectId, info: &ClientInfoRef) {
        let changed = info.change_mask().contains(ClientChangeMask::PROPS);
        if let Some(props) = changed_props(changed, info.props()) {
            self.update(|graph| graph.put_client(id, props));
        }
    }

    fn metadata_property(&self, subject: u32, key: Option<&str>, value: Option<&str>) -> i32 {
        if subject == 0 {
            self.update(|graph| graph.set_default(key, value));
        }
        0
    }
}

struct Remover {
    proxies: Rc<RefCell<Proxies>>,
    watcher: Watcher,
}

impl Remover {
    fn forget(&self, id: ObjectId) {
        self.proxies.borrow_mut().forget(id);
        self.watcher.update(|graph| graph.remove(id));
    }
}

struct Binder {
    registry: RegistryWeak,
    proxies: Rc<RefCell<Proxies>>,
    watcher: Watcher,
}

impl Binder {
    fn bind(&self, global: &GlobalObject<&DictRef>) {
        match global.type_ {
            ObjectType::Node => self.bind_node(global),
            ObjectType::Device => self.bind_device(global),
            ObjectType::Client => self.bind_client(global),
            ObjectType::Link => self.add_link(global),
            ObjectType::Metadata => self.bind_metadata(global),
            _ => {}
        }
    }

    fn bound<T: ProxyT>(&self, global: &GlobalObject<&DictRef>) -> Option<T> {
        self.registry.upgrade()?.bind(global).ok()
    }

    fn keep(&self, id: ObjectId, listener: Box<dyn Listener>) {
        self.proxies.borrow_mut().listeners.insert(id, listener);
    }

    fn bind_node(&self, global: &GlobalObject<&DictRef>) {
        let Some(node) = self.bound::<Node>(global) else {
            return;
        };
        let id = global.id;
        let (on_info, on_param) = (self.watcher.clone(), self.watcher.clone());
        let listener = node
            .add_listener_local()
            .info(move |info| on_info.node_info(id, info))
            .param(move |_, kind, _, _, param| on_param.node_param(id, kind, param))
            .register();
        node.subscribe_params(&[ParamType::Props]);
        self.keep(id, Box::new(listener));
        self.proxies.borrow_mut().nodes.insert(id, node);
    }

    fn bind_device(&self, global: &GlobalObject<&DictRef>) {
        let Some(device) = self.bound::<Device>(global) else {
            return;
        };
        let (id, watcher) = (global.id, self.watcher.clone());
        let listener = device
            .add_listener_local()
            .info(move |info| watcher.device_info(id, info))
            .register();
        self.keep(id, Box::new(listener));
        self.proxies
            .borrow_mut()
            .others
            .insert(id, Box::new(device));
    }

    fn bind_client(&self, global: &GlobalObject<&DictRef>) {
        let Some(client) = self.bound::<Client>(global) else {
            return;
        };
        let (id, watcher) = (global.id, self.watcher.clone());
        let listener = client
            .add_listener_local()
            .info(move |info| watcher.client_info(id, info))
            .register();
        self.keep(id, Box::new(listener));
        self.proxies
            .borrow_mut()
            .others
            .insert(id, Box::new(client));
    }

    fn add_link(&self, global: &GlobalObject<&DictRef>) {
        let Some(link) = global.props.and_then(link_of) else {
            return;
        };
        self.watcher.update(|graph| graph.put_link(global.id, link));
    }

    fn bind_metadata(&self, global: &GlobalObject<&DictRef>) {
        let is_default = global
            .props
            .is_some_and(|props| props.get(METADATA_NAME) == Some(DEFAULT_METADATA));
        let Some(metadata) = is_default.then(|| self.bound::<Metadata>(global)).flatten() else {
            return;
        };
        let watcher = self.watcher.clone();
        let listener = metadata
            .add_listener_local()
            .property(move |subject, key, _, value| watcher.metadata_property(subject, key, value))
            .register();
        self.keep(global.id, Box::new(listener));
        self.proxies.borrow_mut().metadata = Some(metadata);
    }
}

fn refresh_node(graph: &mut Graph, id: ObjectId, props: Option<Props>, running: bool) {
    if let Some(props) = props {
        graph.put_node(id, props);
    }
    graph.set_running(id, running);
}

fn changed_props(changed: bool, dict: Option<&DictRef>) -> Option<Props> {
    dict.filter(|_| changed).map(props_of)
}

fn props_of(dict: &DictRef) -> Props {
    dict.iter().collect()
}

fn link_of(props: &DictRef) -> Option<Link> {
    Some(Link {
        output: props.get(LINK_OUTPUT_NODE)?.parse().ok()?,
        input: props.get(LINK_INPUT_NODE)?.parse().ok()?,
    })
}
