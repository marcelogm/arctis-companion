#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FilterKind {
    Peaking,
    LowShelf,
    HighShelf,
    LowPass,
    HighPass,
}

impl FilterKind {
    pub const ALL: [Self; 5] = [
        Self::Peaking,
        Self::LowShelf,
        Self::HighShelf,
        Self::LowPass,
        Self::HighPass,
    ];

    #[must_use]
    pub const fn has_gain(self) -> bool {
        !matches!(self, Self::LowPass | Self::HighPass)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvelopeError {
    Frequency,
    Gain,
    Q,
    PassGain,
    TooManyFilters,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hertz(u16);

impl Hertz {
    pub const MIN: u16 = 20;
    pub const MAX: u16 = 20_000;

    pub const fn new(hz: u16) -> Result<Self, EnvelopeError> {
        if matches!(hz, Self::MIN..=Self::MAX) {
            Ok(Self(hz))
        } else {
            Err(EnvelopeError::Frequency)
        }
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Gain(i8);

impl Gain {
    pub const MIN_HALF_DB: i8 = -20;
    pub const MAX_HALF_DB: i8 = 20;

    pub const fn from_half_db(half_db: i8) -> Result<Self, EnvelopeError> {
        if matches!(half_db, Self::MIN_HALF_DB..=Self::MAX_HALF_DB) {
            Ok(Self(half_db))
        } else {
            Err(EnvelopeError::Gain)
        }
    }

    #[must_use]
    pub const fn half_db(self) -> i8 {
        self.0
    }

    #[must_use]
    pub fn db(self) -> f64 {
        f64::from(self.0) / 2.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Q(u16);

impl Q {
    pub const MIN_MILLI: u16 = 200;
    pub const MAX_MILLI: u16 = 10_000;

    pub const fn from_milli(milli: u16) -> Result<Self, EnvelopeError> {
        if matches!(milli, Self::MIN_MILLI..=Self::MAX_MILLI) {
            Ok(Self(milli))
        } else {
            Err(EnvelopeError::Q)
        }
    }

    #[must_use]
    pub const fn milli(self) -> u16 {
        self.0
    }

    #[must_use]
    pub fn value(self) -> f64 {
        f64::from(self.0) / 1000.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Filter {
    kind: FilterKind,
    hz: Hertz,
    gain: Gain,
    q: Q,
    enabled: bool,
}

impl Filter {
    pub const fn new(kind: FilterKind, hz: Hertz, gain: Gain, q: Q) -> Result<Self, EnvelopeError> {
        if kind.has_gain() || gain.half_db() == 0 {
            Ok(Self {
                kind,
                hz,
                gain,
                q,
                enabled: true,
            })
        } else {
            Err(EnvelopeError::PassGain)
        }
    }

    pub const fn from_parts(
        kind: FilterKind,
        hz: u16,
        half_db: i8,
        milli_q: u16,
    ) -> Result<Self, EnvelopeError> {
        let hz = match Hertz::new(hz) {
            Ok(hz) => hz,
            Err(error) => return Err(error),
        };
        let gain = match Gain::from_half_db(half_db) {
            Ok(gain) => gain,
            Err(error) => return Err(error),
        };
        let q = match Q::from_milli(milli_q) {
            Ok(q) => q,
            Err(error) => return Err(error),
        };
        Self::new(kind, hz, gain, q)
    }

    #[must_use]
    pub const fn with_enabled(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }

    #[must_use]
    pub const fn kind(self) -> FilterKind {
        self.kind
    }

    #[must_use]
    pub const fn hz(self) -> Hertz {
        self.hz
    }

    #[must_use]
    pub const fn gain(self) -> Gain {
        self.gain
    }

    #[must_use]
    pub const fn q(self) -> Q {
        self.q
    }

    #[must_use]
    pub const fn is_enabled(self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn shapes_sound(self) -> bool {
        self.enabled && !(self.kind.has_gain() && self.gain.half_db() == 0)
    }
}
