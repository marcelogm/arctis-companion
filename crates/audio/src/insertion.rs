use arctis_core::messages::audio::SinkKind;

use crate::eq_names::EQ_SINK_NAME;
use crate::graph::Graph;
use crate::sinks::sink_kind;

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

#[must_use]
pub fn insertion_target(graph: &Graph) -> Option<String> {
    let name = graph.defaults().current()?;
    let id = graph.node_named(name)?;
    (sink_kind(graph, id) == SinkKind::NovaBluetooth).then(|| name.to_owned())
}

#[derive(Debug, Default)]
pub struct Insertion {
    replaced: Option<String>,
}

impl Insertion {
    #[must_use]
    pub fn plan_insert(&self, graph: &Graph) -> InsertStep {
        if self.replaced.is_some() {
            return InsertStep::Retarget;
        }
        insertion_target(graph).map_or(InsertStep::Skip, |replaced| InsertStep::Create { replaced })
    }

    pub fn commit(&mut self, replaced: String) {
        self.replaced = Some(replaced);
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
