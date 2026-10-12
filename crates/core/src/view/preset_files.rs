use super::text::{Arg, Args, Text, preset_arg, saturating_count};
use crate::companion::snapshot::Snapshot;
use crate::eq::apo::{self, ApoError, Clamp, Field, Import};
use crate::eq::curve::{Curve, MAX_FILTERS};
use crate::eq::preset::Preset;
use crate::l10n::{forms, msgid};

const APO_EXTENSION: &str = ".txt";
const JSON_EXTENSION: &str = ".json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Imported {
    pub preset: Preset,
    pub toast: Text,
    pub clamped: Vec<Text>,
}

struct Parsed {
    name: String,
    curve: Curve,
    clamped: Vec<Clamp>,
}

pub fn import_preset(snapshot: &Snapshot, file_name: &str, text: &str) -> Result<Imported, Text> {
    let parsed = if text.trim_start().starts_with('{') {
        from_json(file_name, text)?
    } else {
        from_apo(file_name, text)?
    };
    let id = snapshot.fresh_preset_id(&parsed.name);
    let preset = Preset::custom(id, parsed.name, parsed.curve);
    Ok(Imported {
        toast: imported_toast(&preset, parsed.clamped.len()),
        clamped: parsed.clamped.iter().map(clamp_line).collect(),
        preset,
    })
}

#[must_use]
pub fn export_text(preset: &Preset, file_name: &str) -> Option<String> {
    if file_name.to_ascii_lowercase().ends_with(APO_EXTENSION) {
        Some(apo::render(preset.curve()))
    } else {
        serde_json::to_string_pretty(preset).ok()
    }
}

#[must_use]
pub fn export_file_name(name: &str) -> String {
    format!("{name}{JSON_EXTENSION}")
}

#[must_use]
pub fn export_failed(preset: &Preset) -> Text {
    Text::format(
        msgid("Could not export “{preset}”"),
        vec![("preset", preset_arg(preset))],
    )
}

fn from_json(file_name: &str, text: &str) -> Result<Parsed, Text> {
    let preset: Preset = serde_json::from_str(text).map_err(|_| not_a_preset(file_name))?;
    Ok(Parsed {
        name: name_or_stem(preset.name(), file_name),
        curve: preset.curve().clone(),
        clamped: Vec::new(),
    })
}

fn name_or_stem(name: &str, file_name: &str) -> String {
    let name = name.trim();
    let chosen = if name.is_empty() {
        stem(file_name)
    } else {
        name
    };
    chosen.to_owned()
}

fn from_apo(file_name: &str, text: &str) -> Result<Parsed, Text> {
    let Import { curve, clamped } =
        apo::parse(text).map_err(|error| apo_problem(file_name, error))?;
    Ok(Parsed {
        name: stem(file_name).to_owned(),
        curve,
        clamped,
    })
}

fn stem(file_name: &str) -> &str {
    match file_name.rsplit_once('.') {
        Some((stem, _)) if !stem.trim().is_empty() => stem,
        _ => file_name,
    }
}

fn imported_toast(preset: &Preset, clamped: usize) -> Text {
    let name = vec![("preset", preset_arg(preset))];
    match saturating_count(clamped) {
        0 => Text::format(msgid("Imported “{preset}”"), name),
        n => Text::plural(
            forms(
                "Imported “{preset}”. {n} value was changed to fit the headset",
                "Imported “{preset}”. {n} values were changed to fit the headset",
            ),
            n,
            name,
        ),
    }
}

fn clamp_line(clamp: &Clamp) -> Text {
    let field = match clamp.field {
        Field::Frequency => msgid("Frequency"),
        Field::Gain => msgid("Gain"),
        Field::Q => msgid("Q"),
    };
    Text::format(
        msgid("Filter {filter}: {field}"),
        vec![
            ("filter", Arg::Number(saturating_count(clamp.position + 1))),
            ("field", Arg::Msgid(field.to_owned())),
        ],
    )
}

fn not_a_preset(file_name: &str) -> Text {
    Text::format(msgid("“{file}” is not a preset file"), file_args(file_name))
}

fn apo_problem(file_name: &str, error: ApoError) -> Text {
    match error {
        ApoError::NoFilters => Text::format(msgid("“{file}” has no filters"), file_args(file_name)),
        ApoError::TooManyFilters(found) => too_many_filters(file_name, found),
        ApoError::Malformed { line } => bad_line(
            file_name,
            msgid("Could not read line {line} of “{file}”"),
            line,
        ),
        ApoError::UnsupportedKind { line } => bad_line(
            file_name,
            msgid("Line {line} of “{file}” uses a filter type the headset does not support"),
            line,
        ),
    }
}

fn too_many_filters(file_name: &str, found: usize) -> Text {
    let mut args = file_args(file_name);
    args.push(("max", Arg::Number(saturating_count(MAX_FILTERS))));
    Text::plural(
        forms(
            "“{file}” has {n} filter; the headset takes at most {max}",
            "“{file}” has {n} filters; the headset takes at most {max}",
        ),
        saturating_count(found),
        args,
    )
}

fn bad_line(file_name: &str, msgid: &str, line: usize) -> Text {
    let mut args = file_args(file_name);
    args.push(("line", Arg::Number(saturating_count(line))));
    Text::format(msgid, args)
}

fn file_args(file_name: &str) -> Args {
    vec![("file", Arg::Str(file_name.to_owned()))]
}
