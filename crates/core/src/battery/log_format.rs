use super::sample::{Reading, Sample, UnixSeconds};
use crate::protocol::status::{Charging, Link};
use crate::settings::values::Percent;

pub const RETENTION_SECONDS: u64 = 8 * 24 * 60 * 60;

#[must_use]
pub fn render_line(sample: &Sample) -> String {
    let reading = sample.reading;
    format!(
        "{},{},{},{}\n",
        sample.at.0,
        reading.percent.get(),
        charging_word(reading.charging),
        link_word(reading.link)
    )
}

#[must_use]
pub fn render_log(samples: &[Sample]) -> String {
    samples.iter().map(render_line).collect()
}

#[must_use]
pub fn parse_log(text: &str) -> Vec<Sample> {
    let mut samples: Vec<Sample> = text.lines().filter_map(parse_line).collect();
    samples.sort_by_key(|sample| sample.at);
    samples
}

#[must_use]
pub fn retain_recent(samples: &[Sample], now: UnixSeconds) -> Vec<Sample> {
    let cutoff = now.minus(RETENTION_SECONDS);
    samples
        .iter()
        .filter(|sample| sample.at >= cutoff)
        .copied()
        .collect()
}

fn parse_line(text: &str) -> Option<Sample> {
    let mut fields = text.trim().split(',');
    let at = UnixSeconds(fields.next()?.parse().ok()?);
    let percent = Percent::try_from(fields.next()?.parse::<u8>().ok()?).ok()?;
    let charging = parse_charging(fields.next()?)?;
    let link = parse_link(fields.next()?)?;
    let reading = Reading {
        percent,
        charging,
        link,
    };
    fields.next().is_none().then_some(Sample { at, reading })
}

const fn charging_word(charging: Charging) -> &'static str {
    match charging {
        Charging::Unknown => "unknown",
        Charging::Charging => "charging",
        Charging::PluggedIn => "plugged-in",
        Charging::Discharging => "discharging",
    }
}

const fn link_word(link: Link) -> &'static str {
    match link {
        Link::NotPaired => "not-paired",
        Link::Searching => "searching",
        Link::HeadsetOff => "headset-off",
        Link::Connected => "connected",
    }
}

fn parse_charging(word: &str) -> Option<Charging> {
    Charging::ALL
        .into_iter()
        .find(|charging| charging_word(*charging) == word)
}

fn parse_link(word: &str) -> Option<Link> {
    Link::ALL.into_iter().find(|link| link_word(*link) == word)
}
