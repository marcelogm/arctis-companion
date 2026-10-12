use std::ops::Range;
use std::rc::Rc;

use arctis_core::battery::sample::{Reading, Sample, UnixSeconds};
use arctis_core::companion::Snapshot;
use arctis_core::protocol::status::{Charging, Link};
use arctis_core::testkit::snapshots::{percent, snapshot};
use arctis_ui::{BatteryChart, BatteryPage};
use gtk::prelude::*;
use gtk::{graphene, gsk};

use crate::support::init;

pub const CHART_WIDTH: f64 = 202.0;
pub const CHART_HEIGHT: f64 = 102.0;
pub const PLOT_INSET: f64 = 1.0;
pub const SHADE_ALPHA: f32 = 0.25;
pub const NOW: UnixSeconds = UnixSeconds(1_000_000_000);
const HOUR: u64 = 3_600;
const WEEK_HOURS: u64 = 7 * 24;
const CHARGING_HOURS: u64 = 3;
const FULL: u64 = 100;
const CHARGE_PER_HOUR: u64 = 20;
const DRAIN_PER_HOUR: u64 = 3;
const EMPTIEST: u64 = 31;
const HEADSET_OFF: Range<u64> = 10..12;

pub fn synthetic_week() -> Vec<Sample> {
    (0..=WEEK_HOURS).rev().map(hourly_sample).collect()
}

fn hourly_sample(hours_ago: u64) -> Sample {
    Sample {
        at: NOW.minus(hours_ago * HOUR),
        reading: hourly_reading(hours_ago),
    }
}

fn hourly_reading(hours_ago: u64) -> Reading {
    let hour_of_day = hours_ago % 24;
    let charging = hour_of_day < CHARGING_HOURS;
    Reading {
        percent: percent(level(hour_of_day, charging)),
        charging: if charging {
            Charging::Charging
        } else {
            Charging::Discharging
        },
        link: if HEADSET_OFF.contains(&hours_ago) {
            Link::HeadsetOff
        } else {
            Link::Connected
        },
    }
}

fn level(hour_of_day: u64, charging: bool) -> u8 {
    let level = if charging {
        FULL - hour_of_day * CHARGE_PER_HOUR
    } else {
        EMPTIEST + hour_of_day * DRAIN_PER_HOUR
    };
    u8::try_from(level).expect("a percent")
}

pub fn drawn(chart: &BatteryChart, width: f64, height: f64) -> Vec<gsk::RenderNode> {
    let snapshot = gtk::Snapshot::new();
    chart.draw(&snapshot, width, height);
    snapshot.to_node().map(top_level).unwrap_or_default()
}

pub fn count(nodes: &[gsk::RenderNode], kind: gsk::RenderNodeType) -> usize {
    nodes.iter().filter(|node| node.node_type() == kind).count()
}

pub fn stroke_bounds(nodes: &[gsk::RenderNode]) -> Vec<graphene::Rect> {
    nodes
        .iter()
        .filter_map(|node| node.downcast_ref::<gsk::StrokeNode>())
        .filter_map(|stroke| stroke.path().bounds())
        .collect()
}

fn top_level(node: gsk::RenderNode) -> Vec<gsk::RenderNode> {
    match node.downcast_ref::<gsk::ContainerNode>() {
        Some(container) => (0..container.n_children())
            .map(|at| container.child(at))
            .collect(),
        None => vec![node],
    }
}

pub fn with_history(history: Vec<Sample>) -> Snapshot {
    let mut s = snapshot();
    s.battery.history = history;
    s
}

pub fn shown_battery_page(s: Snapshot) -> BatteryPage {
    init();
    let page = BatteryPage::default();
    page.display_history(Rc::new(s), NOW);
    page
}
