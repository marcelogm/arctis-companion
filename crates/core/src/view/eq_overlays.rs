use super::eq_graph::{
    ColorRole, GraphPoint, HIGH_HZ, LOW_HZ, LevelRange, LineStyle, Series, Weight,
};
use super::text::Text;
use crate::eq::biquad::narrow;
use crate::eq::curve::Curve;
use crate::eq::frequency_response::FrequencyResponse;
use crate::eq::reference::{Overlays, SeriesId, Target, headset_response, headset_response_with};
use crate::l10n::msgid;

pub const OVERLAID: LevelRange = LevelRange {
    low_db: -18.0,
    high_db: 12.0,
};

#[must_use]
pub const fn level_range(overlays: Overlays) -> LevelRange {
    if overlays.any() {
        OVERLAID
    } else {
        LevelRange::RELATIVE
    }
}

#[must_use]
pub fn overlay_series(curve: &Curve, overlays: Overlays) -> Vec<Series> {
    let target = overlays
        .target
        .map(|target| (SeriesId::Target(target), target.response()));
    let headset = overlays.headset.then(|| {
        [
            (SeriesId::Headset, headset_response()),
            (SeriesId::HeadsetWithEq, headset_response_with(curve)),
        ]
    });
    target
        .into_iter()
        .chain(headset.into_iter().flatten())
        .map(|(id, response)| styled(id, &response))
        .collect()
}

#[must_use]
pub fn overlay_points(response: &FrequencyResponse) -> Vec<GraphPoint> {
    let shown = f64::from(LOW_HZ)..=f64::from(HIGH_HZ);
    response
        .points()
        .iter()
        .filter(|point| shown.contains(&point.hz))
        .map(|point| GraphPoint {
            hz: narrow(point.hz),
            db: narrow(point.db),
        })
        .collect()
}

#[must_use]
pub const fn summary(overlays: Overlays) -> Option<&'static str> {
    let text = match (overlays.headset, overlays.target) {
        (false, None) => return None,
        (true, None) => {
            msgid("Showing the measured headset response, with and without the equalizer.")
        }
        (false, Some(Target::Harman)) => msgid("Showing the Harman target (B&K 5128)."),
        (false, Some(Target::DiffuseField)) => {
            msgid("Showing the diffuse field target (B&K 5128).")
        }
        (true, Some(Target::Harman)) => msgid(
            "Showing the measured headset response, with and without the equalizer, and the Harman target (B&K 5128).",
        ),
        (true, Some(Target::DiffuseField)) => msgid(
            "Showing the measured headset response, with and without the equalizer, and the diffuse field target (B&K 5128).",
        ),
    };
    Some(text)
}

fn styled(id: SeriesId, response: &FrequencyResponse) -> Series {
    let (style, role, weight) = match id {
        SeriesId::Headset => (LineStyle::Solid, ColorRole::Foreground, Weight::Faint),
        SeriesId::HeadsetWithEq => (LineStyle::Solid, ColorRole::Accent, Weight::Bold),
        SeriesId::Target(_) => (LineStyle::Dashed, ColorRole::Foreground, Weight::Regular),
    };
    Series {
        points: overlay_points(response),
        style,
        role,
        weight,
        filled: false,
        label: Some(Text::plain(id.label())),
        reference: Some(id),
    }
}
