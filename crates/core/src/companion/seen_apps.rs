use std::iter;

use serde::{Deserialize, Serialize};

use crate::messages::audio::StreamInfo;

pub const MAX_SEEN_APPS: usize = 50;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SeenApp {
    pub key: String,
    pub name: String,
    pub identifiers: Vec<String>,
}

impl SeenApp {
    #[must_use]
    pub fn from_stream(stream: &StreamInfo) -> Option<Self> {
        let key = stream.ids.key()?;
        Some(Self {
            key: key.to_owned(),
            name: stream.name.clone(),
            identifiers: stream.ids.as_slice().to_vec(),
        })
    }

    #[must_use]
    pub fn answers_to(&self, identifier: &str) -> bool {
        self.identifiers
            .iter()
            .any(|known| known.eq_ignore_ascii_case(identifier))
    }
}

#[must_use]
pub fn remember(apps: &[SeenApp], app: SeenApp) -> Vec<SeenApp> {
    let key = app.key.clone();
    let older = apps.iter().filter(|known| known.key != key).cloned();
    iter::once(app).chain(older).take(MAX_SEEN_APPS).collect()
}

#[must_use]
pub fn newly_playing(before: &[StreamInfo], after: &[StreamInfo]) -> Vec<SeenApp> {
    let mut started: Vec<SeenApp> = Vec::new();
    for app in playing(after) {
        let known = playing(before).any(|old| old.key == app.key)
            || started.iter().any(|new| new.key == app.key);
        if !known {
            started.push(app);
        }
    }
    started
}

fn playing(streams: &[StreamInfo]) -> impl Iterator<Item = SeenApp> + '_ {
    streams
        .iter()
        .filter(|stream| stream.running)
        .filter_map(SeenApp::from_stream)
}
