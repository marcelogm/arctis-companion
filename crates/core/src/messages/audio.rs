use crate::eq::Curve;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioCommand {
    InsertEqualizer(Curve),
    Retarget(Curve),
    RemoveEqualizer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioEvent {
    Available,
    Unavailable,
    DefaultSink(DefaultSink),
    Streams(Vec<StreamInfo>),
    ManagerDenied,
    BluetoothNodeSeen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundPath {
    Dongle,
    Bluetooth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SinkKind {
    NovaBluetooth,
    NovaDongle,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefaultSink {
    pub terminal: SinkKind,
    pub via_equalizer: bool,
    pub via_easy_effects: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamInfo {
    pub ids: AppIdentifiers,
    pub name: String,
    pub running: bool,
    pub path: Option<SoundPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppIdentifiers(Vec<String>);

impl AppIdentifiers {
    #[must_use]
    pub fn new(
        portal_app_id: Option<&str>,
        application_id: Option<&str>,
        binary: Option<&str>,
    ) -> Self {
        let binary = binary.map(str::to_lowercase);
        let mut ids = Self(Vec::new());
        for id in [portal_app_id, application_id, binary.as_deref()]
            .into_iter()
            .flatten()
        {
            ids.push_new(id);
        }
        ids
    }

    fn push_new(&mut self, id: &str) {
        if !id.is_empty() && !self.0.iter().any(|known| known == id) {
            self.0.push(id.to_owned());
        }
    }

    #[must_use]
    pub fn key(&self) -> Option<&str> {
        self.0.first().map(String::as_str)
    }

    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}
