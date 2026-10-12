use std::cell::RefCell;
use std::rc::{Rc, Weak};

use arctis_core::eq::Curve;
use pipewire::core::CoreRc;
use pipewire::spa::pod::Pod;

use crate::eq_sink::{EqSink, OnVolume, Replaced};
use crate::outputs::{EQ_SINK_NAME, InsertStep, Outputs, RemoveStep};
use crate::registry::Registry;
use crate::volume::{VolumeChange, VolumeMirror, props_pod};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inserted {
    Created,
    Unchanged,
}

pub struct SystemEngine {
    core: CoreRc,
    registry: Rc<Registry>,
    outputs: RefCell<Outputs>,
    sink: RefCell<Option<EqSink>>,
    mirror: Rc<RefCell<VolumeMirror>>,
}

impl SystemEngine {
    #[must_use]
    pub fn new(core: CoreRc, registry: Rc<Registry>) -> Self {
        Self {
            core,
            registry,
            outputs: RefCell::default(),
            sink: RefCell::default(),
            mirror: Rc::default(),
        }
    }

    pub fn insert(&self, curve: &Curve) -> Inserted {
        let step = self.outputs.borrow_mut().insert(&self.registry.graph());
        match step {
            InsertStep::Create { replaced } => self.create(curve, &replaced),
            InsertStep::Retarget => {
                self.retarget(curve);
                Inserted::Unchanged
            }
            InsertStep::Skip => Inserted::Unchanged,
        }
    }

    fn create(&self, curve: &Curve, replaced: &str) -> Inserted {
        let forwarder = forwarder(Rc::downgrade(&self.registry), self.mirror.clone(), replaced);
        let replaced = Replaced {
            name: replaced.to_owned(),
            volume: self.registry.graph().volume_of(replaced),
        };
        let Ok(sink) = EqSink::create(&self.core, curve, replaced, forwarder) else {
            self.outputs.borrow_mut().remove(&self.registry.graph());
            return Inserted::Unchanged;
        };
        *self.sink.borrow_mut() = Some(sink);
        self.registry.write_default_sink(EQ_SINK_NAME);
        Inserted::Created
    }

    pub fn retarget(&self, curve: &Curve) {
        if let Some(sink) = self.sink.borrow().as_ref() {
            sink.retarget(curve);
        }
    }

    pub fn remove(&self) {
        let step = self.outputs.borrow_mut().remove(&self.registry.graph());
        if let RemoveStep::Restore(replaced) = &step {
            self.registry.write_default_sink(replaced);
        }
        let removed = self.sink.borrow_mut().take();
        drop(removed);
        *self.mirror.borrow_mut() = VolumeMirror::default();
    }

    #[must_use]
    pub fn manager_denied(&self) -> bool {
        self.outputs.borrow().manager_denied(&self.registry.graph())
    }
}

fn forwarder(
    registry: Weak<Registry>,
    mirror: Rc<RefCell<VolumeMirror>>,
    target: &str,
) -> OnVolume {
    let target = target.to_owned();
    Rc::new(move |change: VolumeChange| {
        if let Some(registry) = registry.upgrade() {
            forward(&registry, &mirror, &target, change);
        }
    })
}

fn forward(
    registry: &Registry,
    mirror: &RefCell<VolumeMirror>,
    target: &str,
    change: VolumeChange,
) {
    let Some(volume) = mirror.borrow_mut().apply(change) else {
        return;
    };
    let bytes = props_pod(&volume);
    if let Some(pod) = Pod::from_bytes(&bytes) {
        registry.set_node_props(target, pod);
    }
}
