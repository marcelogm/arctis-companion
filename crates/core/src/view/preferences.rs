use super::text::Text;
use crate::companion::snapshot::Snapshot;
use crate::l10n::msgid;

pub const LOW_BATTERY_MIN: u8 = 5;
pub const LOW_BATTERY_MAX: u8 = 50;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Background {
    #[default]
    Off,
    On,
    Denied,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferencesModel {
    pub run_in_background: bool,
    pub background_subtitle: Option<Text>,
    pub low_battery: u8,
}

#[must_use]
pub fn preferences(snapshot: &Snapshot, background: Background) -> PreferencesModel {
    PreferencesModel {
        run_in_background: background == Background::On,
        background_subtitle: (background == Background::Denied)
            .then(|| Text::plain(msgid("Allow it in Settings → Apps → Arctis Companion"))),
        low_battery: snapshot
            .battery
            .low_threshold
            .get()
            .clamp(LOW_BATTERY_MIN, LOW_BATTERY_MAX),
    }
}
