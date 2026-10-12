use crate::eq::curve::Curve;
use crate::messages::audio::{AudioEvent, DefaultSink, SinkKind, SoundPath, StreamInfo};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Route {
    Dongle,
    Bluetooth,
    None,
}

impl Route {
    #[must_use]
    pub const fn path(self) -> Option<SoundPath> {
        match self {
            Self::Dongle => Some(SoundPath::Dongle),
            Self::Bluetooth => Some(SoundPath::Bluetooth),
            Self::None => None,
        }
    }
}

#[must_use]
pub const fn route(default_sink: Option<DefaultSink>) -> Route {
    match default_sink {
        Some(DefaultSink {
            terminal: SinkKind::NovaBluetooth,
            ..
        }) => Route::Bluetooth,
        Some(DefaultSink {
            terminal: SinkKind::NovaDongle,
            ..
        }) => Route::Dongle,
        _ => Route::None,
    }
}

#[must_use]
pub const fn needs_equalizer(sink: DefaultSink) -> bool {
    matches!(sink.terminal, SinkKind::NovaBluetooth)
        && !sink.via_equalizer
        && !sink.via_easy_effects
}

#[must_use]
pub const fn easy_effects_active(default_sink: Option<DefaultSink>) -> bool {
    matches!(default_sink, Some(sink) if sink.via_easy_effects)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AudioService {
    Unknown,
    Available,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioState {
    pub service: AudioService,
    pub default_sink: Option<DefaultSink>,
    pub streams: Vec<StreamInfo>,
    pub manager_denied: bool,
    pub equalizer: Option<Curve>,
}

impl AudioState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            service: AudioService::Unknown,
            default_sink: None,
            streams: Vec::new(),
            manager_denied: false,
            equalizer: None,
        }
    }

    #[must_use]
    pub const fn route(&self) -> Route {
        route(self.default_sink)
    }

    #[must_use]
    pub const fn easy_effects_active(&self) -> bool {
        easy_effects_active(self.default_sink)
    }

    pub fn apply(&mut self, event: &AudioEvent) {
        match event {
            AudioEvent::Available => self.service = AudioService::Available,
            AudioEvent::Unavailable => *self = Self::unavailable(),
            AudioEvent::DefaultSink(sink) => self.default_sink = Some(*sink),
            AudioEvent::Streams(streams) => self.streams.clone_from(streams),
            AudioEvent::ManagerDenied => self.manager_denied = true,
            AudioEvent::EqualizerNotInserted => self.equalizer = None,
            AudioEvent::BluetoothNodeSeen => {}
        }
    }

    fn unavailable() -> Self {
        Self {
            service: AudioService::Unavailable,
            ..Self::new()
        }
    }
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new()
    }
}
