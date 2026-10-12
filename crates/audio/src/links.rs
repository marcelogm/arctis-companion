use crate::graph::{AUDIO_SINK, Graph, INPUT_STREAM, OUTPUT_STREAM, ObjectId};
use crate::keys::LINK_GROUP;

pub const MAX_HOPS: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chain {
    nodes: Vec<ObjectId>,
    terminal: ObjectId,
}

impl Chain {
    #[must_use]
    pub fn nodes(&self) -> &[ObjectId] {
        &self.nodes
    }

    #[must_use]
    pub fn terminal(&self) -> ObjectId {
        self.terminal
    }
}

#[must_use]
pub fn follow(graph: &Graph, start: ObjectId) -> Option<Chain> {
    let mut walk = Walk {
        nodes: vec![start],
        hops: 0,
    };
    loop {
        match next_hop(graph, walk.current()) {
            Hop::WithinFilter(next) => walk.enter(next, 0)?,
            Hop::Link(next) => walk.enter(next, 1)?,
            Hop::End => return walk.finish(graph),
        }
    }
}

enum Hop {
    WithinFilter(ObjectId),
    Link(ObjectId),
    End,
}

struct Walk {
    nodes: Vec<ObjectId>,
    hops: usize,
}

impl Walk {
    fn current(&self) -> ObjectId {
        self.nodes.last().copied().unwrap_or_default()
    }

    fn enter(&mut self, next: ObjectId, cost: usize) -> Option<()> {
        let fresh = !self.nodes.contains(&next);
        self.hops += cost;
        self.nodes.push(next);
        (fresh && self.hops <= MAX_HOPS).then_some(())
    }

    fn finish(self, graph: &Graph) -> Option<Chain> {
        let terminal = self.current();
        graph.node(terminal)?.is(AUDIO_SINK).then_some(Chain {
            nodes: self.nodes,
            terminal,
        })
    }
}

fn next_hop(graph: &Graph, id: ObjectId) -> Hop {
    if let Some(output) = filter_output(graph, id) {
        return Hop::WithinFilter(output);
    }
    graph
        .inputs_fed_by(id)
        .filter(|input| carries_audio_on(graph, *input))
        .min()
        .map_or(Hop::End, Hop::Link)
}

fn filter_output(graph: &Graph, id: ObjectId) -> Option<ObjectId> {
    let sink = graph.node(id).filter(|node| node.is(AUDIO_SINK))?;
    let group = sink.props.get(LINK_GROUP)?;
    graph
        .nodes()
        .find(|(other, node)| {
            *other != id && node.is(OUTPUT_STREAM) && node.props.get(LINK_GROUP) == Some(group)
        })
        .map(|(other, _)| other)
}

fn carries_audio_on(graph: &Graph, id: ObjectId) -> bool {
    graph.node(id).is_some_and(|node| !node.is(INPUT_STREAM))
}
