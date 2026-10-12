use super::adoption::AdoptionStatus;
use super::app_settings::AppRules;
use super::catalog::Catalog;
use super::deadlines::earliest;
use super::per_path::PerPath;
use super::routing::Route;
use super::timing::{RULE_ENTER, RULE_LEAVE};
use crate::eq::preset::PresetId;
use crate::messages::audio::{SoundPath, StreamInfo};
use crate::time::Millis;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PresetReason {
    Route,
    App(String),
    Manual(Option<String>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActivePreset {
    pub preset: PresetId,
    pub reason: PresetReason,
}

impl ActivePreset {
    #[must_use]
    pub const fn route(preset: PresetId) -> Self {
        Self {
            preset,
            reason: PresetReason::Route,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Candidate {
    pub key: String,
    pub name: String,
    pub preset: PresetId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Member {
    pub candidate: Candidate,
    pub joined: Millis,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuledSet {
    members: Vec<Member>,
    arriving: Vec<(Candidate, Millis)>,
    leaving: Vec<(String, Millis)>,
}

impl RuledSet {
    pub fn observe(&mut self, running: &[Candidate], now: Millis) {
        self.drop_stopped_arrivals(running);
        self.cancel_resumed_departures(running);
        self.track_newcomers(running, now);
        self.refresh(running);
        self.start_leaving(running, now);
    }

    pub fn advance(&mut self, now: Millis) -> bool {
        let joined = self.join_due(now);
        let left = self.leave_due(now);
        joined || left
    }

    #[must_use]
    pub fn next_deadline(&self) -> Option<Millis> {
        let joins = self
            .arriving
            .iter()
            .map(|(_, since)| since.after(RULE_ENTER));
        let leaves = self
            .leaving
            .iter()
            .map(|(_, since)| since.after(RULE_LEAVE));
        joins.chain(leaves).min()
    }

    #[must_use]
    pub fn newest(&self) -> Option<&Member> {
        self.members.iter().max_by_key(|member| member.joined)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub fn clear(&mut self) -> bool {
        let changed = !self.members.is_empty();
        *self = Self::default();
        changed
    }

    fn drop_stopped_arrivals(&mut self, running: &[Candidate]) {
        self.arriving
            .retain(|(waiting, _)| is_running(running, &waiting.key));
    }

    fn cancel_resumed_departures(&mut self, running: &[Candidate]) {
        self.leaving.retain(|(key, _)| !is_running(running, key));
    }

    fn track_newcomers(&mut self, running: &[Candidate], now: Millis) {
        let newcomers: Vec<(Candidate, Millis)> = running
            .iter()
            .filter(|candidate| !self.is_known(&candidate.key))
            .map(|candidate| (candidate.clone(), now))
            .collect();
        self.arriving.extend(newcomers);
    }

    fn is_known(&self, key: &str) -> bool {
        self.is_member(key) || self.arriving.iter().any(|(waiting, _)| waiting.key == key)
    }

    fn is_member(&self, key: &str) -> bool {
        self.members
            .iter()
            .any(|member| member.candidate.key == key)
    }

    fn is_leaving(&self, key: &str) -> bool {
        self.leaving.iter().any(|(gone, _)| gone == key)
    }

    fn start_leaving(&mut self, running: &[Candidate], now: Millis) {
        let silent: Vec<String> = self
            .members
            .iter()
            .map(|member| member.candidate.key.clone())
            .filter(|key| !is_running(running, key) && !self.is_leaving(key))
            .collect();
        self.leaving
            .extend(silent.into_iter().map(|key| (key, now)));
    }

    fn refresh(&mut self, running: &[Candidate]) {
        for member in &mut self.members {
            let current = running.iter().find(|c| c.key == member.candidate.key);
            member.candidate = current.unwrap_or(&member.candidate).clone();
        }
    }

    fn join_due(&mut self, now: Millis) -> bool {
        let (due, waiting): (Vec<_>, Vec<_>) = self
            .arriving
            .drain(..)
            .partition(|(_, since)| since.after(RULE_ENTER) <= now);
        self.arriving = waiting;
        let changed = !due.is_empty();
        self.members
            .extend(due.into_iter().map(|(candidate, since)| Member {
                candidate,
                joined: since.after(RULE_ENTER),
            }));
        changed
    }

    fn leave_due(&mut self, now: Millis) -> bool {
        let (due, waiting): (Vec<_>, Vec<_>) = self
            .leaving
            .drain(..)
            .partition(|(_, since)| since.after(RULE_LEAVE) <= now);
        self.leaving = waiting;
        self.members
            .retain(|member| !due.iter().any(|(key, _)| *key == member.candidate.key));
        !due.is_empty()
    }
}

fn is_running(running: &[Candidate], key: &str) -> bool {
    running.iter().any(|candidate| candidate.key == key)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ManualPick {
    pub preset: PresetId,
    pub preview: bool,
}

impl ManualPick {
    #[must_use]
    pub const fn lasting(preset: PresetId) -> Self {
        Self {
            preset,
            preview: false,
        }
    }

    #[must_use]
    pub const fn preview(preset: PresetId) -> Self {
        Self {
            preset,
            preview: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PathSelection {
    pub ruled: RuledSet,
    pub manual: Option<ManualPick>,
}

impl PathSelection {
    #[must_use]
    pub fn active(&self, route_preset: &PresetId) -> ActivePreset {
        let newest = self.ruled.newest();
        match (&self.manual, newest) {
            (Some(pick), _) => ActivePreset {
                preset: pick.preset.clone(),
                reason: PresetReason::Manual(newest.map(|member| member.candidate.name.clone())),
            },
            (None, Some(member)) => ActivePreset {
                preset: member.candidate.preset.clone(),
                reason: PresetReason::App(member.candidate.name.clone()),
            },
            (None, None) => ActivePreset::route(route_preset.clone()),
        }
    }

    pub fn observe(&mut self, running: &[Candidate], now: Millis) {
        self.ruled.observe(running, now);
    }

    pub fn follow(&mut self, running: Option<&[Candidate]>, now: Millis) {
        let Some(running) = running else {
            self.clear();
            return;
        };
        self.observe(running, now);
        self.advance(now);
    }

    pub fn advance(&mut self, now: Millis) {
        if self.ruled.advance(now) {
            self.manual = None;
        }
    }

    pub fn clear(&mut self) {
        if self.ruled.clear() {
            self.manual = None;
        }
    }

    pub fn end_preview(&mut self) {
        if self.manual.as_ref().is_some_and(|pick| pick.preview) {
            self.manual = None;
        }
    }
}

pub type Selection = PerPath<PathSelection>;

impl Selection {
    pub fn end_manual_picks(&mut self) {
        for selection in self.values_mut() {
            selection.manual = None;
        }
    }

    pub fn end_previews(&mut self) {
        for selection in self.values_mut() {
            selection.end_preview();
        }
    }

    #[must_use]
    pub fn next_deadline(&self) -> Option<Millis> {
        earliest(
            self.values()
                .map(|selection| selection.ruled.next_deadline()),
        )
    }
}

#[must_use]
pub fn candidates(
    streams: &[StreamInfo],
    rules: &AppRules,
    catalog: &Catalog,
    path: SoundPath,
) -> Vec<Candidate> {
    let mut found: Vec<Candidate> = Vec::new();
    let playing = streams
        .iter()
        .filter(|stream| stream.running && stream.path == Some(path));
    for candidate in playing.filter_map(|stream| candidate(stream, rules, catalog)) {
        if !found.iter().any(|known| known.key == candidate.key) {
            found.push(candidate);
        }
    }
    found
}

fn candidate(stream: &StreamInfo, rules: &AppRules, catalog: &Catalog) -> Option<Candidate> {
    let key = stream.ids.key()?;
    let preset = rules
        .matching(&stream.ids)
        .filter(|preset| catalog.contains(preset))?;
    Some(Candidate {
        key: key.to_owned(),
        name: stream.name.clone(),
        preset: preset.clone(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActivateError {
    NoRoute,
    NotReady,
}

pub fn picked_path(
    route: Route,
    dongle_present: bool,
    adoption: AdoptionStatus,
) -> Result<SoundPath, ActivateError> {
    let path = route
        .path()
        .or(dongle_present.then_some(SoundPath::Dongle))
        .ok_or(ActivateError::NoRoute)?;
    if path == SoundPath::Dongle && adoption != AdoptionStatus::Adopted {
        return Err(ActivateError::NotReady);
    }
    Ok(path)
}
