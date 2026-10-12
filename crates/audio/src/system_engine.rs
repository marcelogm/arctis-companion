use std::cell::RefCell;
use std::rc::{Rc, Weak};

use arctis_core::eq::Curve;
use pipewire::core::CoreRc;
use pipewire::spa::pod::Pod;

use crate::eq_names::EQ_SINK_NAME;
use crate::eq_sink::{EqSink, OnVolume, Replaced};
use crate::insertion::{InsertStep, Insertion, RemoveStep};
use crate::registry::Registry;
use crate::volume::{VolumeChange, VolumeMirror, props_pod};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inserted {
    Created,
    NotCreated,
}

pub struct SystemEngine {
    core: CoreRc,
    registry: Rc<Registry>,
    insertion: RefCell<Insertion>,
    sink: RefCell<Option<EqSink>>,
    mirror: Rc<RefCell<VolumeMirror>>,
}

impl SystemEngine {
    #[must_use]
    pub fn new(core: CoreRc, registry: Rc<Registry>) -> Self {
        Self {
            core,
            registry,
            insertion: RefCell::default(),
            sink: RefCell::default(),
            mirror: Rc::default(),
        }
    }

    pub fn insert(&self, curve: &Curve) -> Inserted {
        let step = self.insertion.borrow().plan_insert(&self.registry.graph());
        match step {
            InsertStep::Create { replaced } => self.create(curve, replaced),
            InsertStep::Retarget => {
                self.retarget(curve);
                Inserted::NotCreated
            }
            InsertStep::Skip => Inserted::NotCreated,
        }
    }

    fn create(&self, curve: &Curve, replaced: String) -> Inserted {
        let forwarder = forwarder(
            Rc::downgrade(&self.registry),
            self.mirror.clone(),
            &replaced,
        );
        let volume = self.registry.graph().volume_of(&replaced);
        let target = Replaced {
            name: replaced.clone(),
            volume,
        };
        let Ok(sink) = EqSink::create(&self.core, curve, target, forwarder) else {
            return Inserted::NotCreated;
        };
        *self.sink.borrow_mut() = Some(sink);
        self.insertion.borrow_mut().commit(replaced);
        self.registry.write_default_sink(EQ_SINK_NAME);
        Inserted::Created
    }

    pub fn retarget(&self, curve: &Curve) {
        if let Some(sink) = self.sink.borrow().as_ref() {
            sink.retarget(curve);
        }
    }

    pub fn remove(&self) -> RemoveStep {
        let step = self.insertion.borrow_mut().remove(&self.registry.graph());
        if let RemoveStep::Restore(replaced) = &step {
            self.registry.write_default_sink(replaced);
        }
        drop(self.sink.take());
        *self.mirror.borrow_mut() = VolumeMirror::default();
        step
    }

    #[must_use]
    pub fn manager_denied(&self) -> bool {
        self.insertion
            .borrow()
            .manager_denied(&self.registry.graph())
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
