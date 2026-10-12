use arctis_audio::graph::{
    CONFIGURED_AUDIO_SINK, DEFAULT_AUDIO_SINK, Graph, Link, ObjectId, Props,
};
use arctis_audio::outputs::{EQ_LINK_GROUP, EQ_OUTPUT_NAME, EQ_SINK_NAME};

pub const BLUEZ: &str = "bluez_output.AA_BB_CC_DD_EE_FF.1";
pub const USB: &str = "alsa_output.usb-SteelSeries_SteelSeries_Arctis_Nova_5-00.iec958-stereo";
pub const SPEAKERS: &str = "alsa_output.pci-0000_00_1f.3.analog-stereo";

pub fn props(pairs: &[(&str, &str)]) -> Props {
    pairs.iter().copied().collect()
}

pub fn quoted(name: &str) -> String {
    format!("{{\"name\":\"{name}\"}}")
}

pub struct FilterNodes<'a> {
    pub sink: ObjectId,
    pub output: ObjectId,
    pub name: &'a str,
    pub output_name: &'a str,
    pub group: &'a str,
}

pub struct Scene {
    graph: Graph,
    next_link: ObjectId,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            graph: Graph::default(),
            next_link: 1000,
        }
    }
}

impl Scene {
    pub fn node(mut self, id: ObjectId, pairs: &[(&str, &str)]) -> Self {
        self.graph.put_node(id, props(pairs));
        self
    }

    pub fn sink(self, id: ObjectId, name: &str) -> Self {
        self.node(id, &[("media.class", "Audio/Sink"), ("node.name", name)])
    }

    pub fn bluez(self, id: ObjectId) -> Self {
        self.node(
            id,
            &[
                ("media.class", "Audio/Sink"),
                ("node.name", BLUEZ),
                ("node.description", "Arctis Nova 5"),
                ("device.api", "bluez5"),
            ],
        )
    }

    pub fn usb(mut self, id: ObjectId, device: ObjectId, product: &str) -> Self {
        let device_id = device.to_string();
        self.graph.put_device(
            device,
            props(&[
                ("device.vendor.id", "0x1038"),
                ("device.product.id", product),
            ]),
        );
        self.node(
            id,
            &[
                ("media.class", "Audio/Sink"),
                ("node.name", USB),
                ("device.id", &device_id),
            ],
        )
    }

    pub fn speakers(self, id: ObjectId) -> Self {
        self.sink(id, SPEAKERS)
    }

    pub fn equalizer(self, sink: ObjectId, output: ObjectId) -> Self {
        self.filter(&FilterNodes {
            sink,
            output,
            name: EQ_SINK_NAME,
            output_name: EQ_OUTPUT_NAME,
            group: EQ_LINK_GROUP,
        })
    }

    pub fn filter(self, nodes: &FilterNodes) -> Self {
        self.node(
            nodes.sink,
            &[
                ("media.class", "Audio/Sink"),
                ("node.name", nodes.name),
                ("node.link-group", nodes.group),
            ],
        )
        .node(
            nodes.output,
            &[
                ("media.class", "Stream/Output/Audio"),
                ("node.name", nodes.output_name),
                ("node.link-group", nodes.group),
            ],
        )
    }

    pub fn stream(self, id: ObjectId, pairs: &[(&str, &str)]) -> Self {
        let mut all = vec![("media.class", "Stream/Output/Audio")];
        all.extend_from_slice(pairs);
        self.node(id, &all)
    }

    pub fn client(mut self, id: ObjectId, pairs: &[(&str, &str)]) -> Self {
        self.graph.put_client(id, props(pairs));
        self
    }

    pub fn running(mut self, id: ObjectId) -> Self {
        self.graph.set_running(id, true);
        self
    }

    pub fn link(mut self, output: ObjectId, input: ObjectId) -> Self {
        self.graph.put_link(self.next_link, Link { output, input });
        self.next_link += 1;
        self
    }

    pub fn default_sink(self, name: &str) -> Self {
        self.metadata(DEFAULT_AUDIO_SINK, name)
            .metadata(CONFIGURED_AUDIO_SINK, name)
    }

    pub fn metadata(mut self, key: &str, name: &str) -> Self {
        self.graph.set_default(Some(key), Some(&quoted(name)));
        self
    }

    pub fn build(self) -> Graph {
        self.graph
    }
}
