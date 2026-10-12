use arctis_audio::graph::Graph;
use arctis_audio::links::{MAX_HOPS, follow};

use crate::support::{FilterNodes, Scene};

const FIRST: FilterNodes = FilterNodes {
    sink: 20,
    output: 21,
    name: "first_filter",
    output_name: "first_filter_out",
    group: "group-1",
};
const SECOND: FilterNodes = FilterNodes {
    sink: 22,
    output: 23,
    name: "second_filter",
    output_name: "second_filter_out",
    group: "group-2",
};
const THIRD: FilterNodes = FilterNodes {
    sink: 24,
    output: 25,
    name: "third_filter",
    output_name: "third_filter_out",
    group: "group-3",
};

#[test]
fn a_sink_alone_is_its_own_terminal() {
    let graph = Scene::default().speakers(5).build();
    let chain = follow(&graph, 5).expect("chain");
    assert_eq!(chain.nodes(), [5]);
    assert_eq!(chain.terminal(), 5);
}

#[test]
fn a_stream_reaches_the_sink_its_ports_are_linked_to() {
    let graph = Scene::default()
        .speakers(5)
        .stream(11, &[])
        .link(11, 5)
        .link(11, 5)
        .build();
    assert_eq!(follow(&graph, 11).expect("chain").nodes(), [11, 5]);
}

#[test]
fn a_filter_is_crossed_through_its_link_group() {
    let graph = Scene::default()
        .bluez(4)
        .filter(&FIRST)
        .link(21, 4)
        .stream(11, &[])
        .link(11, 20)
        .build();
    let chain = follow(&graph, 11).expect("chain");
    assert_eq!(chain.nodes(), [11, 20, 21, 4]);
    assert_eq!(chain.terminal(), 4);
}

fn easy_effects_chain(plugins: u32) -> Graph {
    let mut scene = Scene::default()
        .usb(7, 3, "0x2232")
        .sink(40, "easyeffects_sink")
        .stream(11, &[])
        .link(11, 40);
    let mut previous = 40;
    for plugin in 0..plugins {
        let id = 41 + plugin;
        scene = scene
            .node(id, &[("node.name", "ee_soe_plugin")])
            .link(previous, id);
        previous = id;
    }
    scene.link(previous, 7).build()
}

fn plugins_for(hops: usize) -> u32 {
    u32::try_from(hops - 2).expect("a small hop limit")
}

#[test]
fn an_easy_effects_chain_within_the_hop_limit_is_followed() {
    let chain = easy_effects_chain(plugins_for(MAX_HOPS));
    assert_eq!(follow(&chain, 11).expect("chain").terminal(), 7);
}

#[test]
fn a_chain_longer_than_the_hop_limit_is_not_followed() {
    let chain = easy_effects_chain(plugins_for(MAX_HOPS + 1));
    assert_eq!(follow(&chain, 11), None);
}

#[test]
fn moving_inside_a_filter_costs_no_hop() {
    let graph = Scene::default()
        .bluez(4)
        .filter(&FIRST)
        .filter(&SECOND)
        .filter(&THIRD)
        .stream(11, &[])
        .link(11, 20)
        .link(21, 22)
        .link(23, 24)
        .link(25, 4)
        .build();
    assert_eq!(follow(&graph, 11).expect("chain").terminal(), 4);
}

#[test]
fn recorders_of_a_sink_monitor_are_not_followed() {
    let graph = Scene::default()
        .speakers(5)
        .node(60, &[("media.class", "Stream/Input/Audio")])
        .link(5, 60)
        .build();
    assert_eq!(follow(&graph, 5).expect("chain").terminal(), 5);
}

#[test]
fn a_loop_gives_no_chain() {
    let graph = Scene::default()
        .node(1, &[("media.class", "Audio/Sink")])
        .node(2, &[("media.class", "Audio/Sink")])
        .link(1, 2)
        .link(2, 1)
        .build();
    assert_eq!(follow(&graph, 1), None);
}

#[test]
fn a_walk_that_ends_away_from_a_sink_gives_no_chain() {
    let graph = Scene::default().stream(11, &[]).build();
    assert_eq!(follow(&graph, 11), None);
    assert_eq!(follow(&graph, 99), None);
}
