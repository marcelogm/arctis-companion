use super::presets::{PresetItem, preset_items};
use super::status_page::StatusPageModel;
use super::text::Text;
use crate::companion::snapshot::Snapshot;
use crate::eq::preset::PresetId;
use crate::l10n::msgid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleRowModel {
    pub identifier: String,
    pub title: Text,
    pub subtitle: Text,
    pub preset: Option<PresetId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppPresetsContent {
    Empty(Box<StatusPageModel>),
    Rules(Vec<RuleRowModel>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppPresetsModel {
    pub description: Text,
    pub automatic_off: Option<Text>,
    pub presets: Vec<PresetItem>,
    pub content: AppPresetsContent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeenRowModel {
    pub identifier: String,
    pub title: Text,
    pub subtitle: Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddRuleModel {
    pub seen: Vec<SeenRowModel>,
    pub other_title: Text,
    pub other_hint: Text,
}

#[must_use]
pub fn app_presets(snapshot: &Snapshot) -> AppPresetsModel {
    let rows: Vec<RuleRowModel> = snapshot
        .presets
        .rules
        .entries()
        .map(|(identifier, preset)| rule_row(snapshot, identifier, preset))
        .collect();
    let content = if rows.is_empty() {
        AppPresetsContent::Empty(Box::new(no_rules()))
    } else {
        AppPresetsContent::Rules(rows)
    };
    AppPresetsModel {
        description: Text::plain(msgid(
            "On the 2.4 GHz connection the headset keeps an app's preset if this computer stops unexpectedly",
        )),
        automatic_off: (!snapshot.presets.automatic)
            .then(|| Text::plain(msgid("Automatic App Presets is off"))),
        presets: preset_items(snapshot),
        content,
    }
}

#[must_use]
pub fn add_rule_dialog(snapshot: &Snapshot) -> AddRuleModel {
    let rules = &snapshot.presets.rules;
    let unruled = snapshot.presets.seen_apps.iter().filter(|app| {
        !app.identifiers
            .iter()
            .any(|identifier| rules.preset_for(identifier).is_some())
    });
    AddRuleModel {
        seen: unruled
            .map(|app| SeenRowModel {
                identifier: app.key.clone(),
                title: Text::verbatim(&app.name),
                subtitle: Text::verbatim(&app.key),
            })
            .collect(),
        other_title: Text::plain(msgid("Other App…")),
        other_hint: Text::plain(msgid("Flatpak app id, application id or binary name")),
    }
}

fn rule_row(snapshot: &Snapshot, identifier: &str, preset: &PresetId) -> RuleRowModel {
    let known = snapshot.presets.find(preset).is_some();
    let name = snapshot
        .presets
        .seen_apps
        .iter()
        .find(|app| app.answers_to(identifier))
        .map_or(identifier, |app| app.name.as_str());
    RuleRowModel {
        identifier: identifier.to_owned(),
        title: Text::verbatim(name),
        subtitle: if known {
            Text::verbatim(identifier)
        } else {
            Text::plain(msgid("Preset missing"))
        },
        preset: known.then(|| preset.clone()),
    }
}

fn no_rules() -> StatusPageModel {
    StatusPageModel::new(
        "applications-multimedia-symbolic",
        msgid("No App Presets"),
        msgid("Choose a preset that applies while an app plays sound"),
        Some(msgid("Add Rule")),
    )
}
