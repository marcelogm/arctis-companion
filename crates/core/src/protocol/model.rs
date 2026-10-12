use crate::settings::headset_settings::Setting;

pub const VENDOR_ID: u16 = 0x1038;
pub const BOOTLOADER_PIDS: [u16; 3] = [0x2233, 0x2254, 0x2265];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Support {
    Verified,
    Untested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Capabilities {
    settings: &'static [Setting],
}

impl Capabilities {
    pub const ALL_SETTINGS: Self = Self::new(&Setting::ALL);

    #[must_use]
    pub const fn new(settings: &'static [Setting]) -> Self {
        Self { settings }
    }

    #[must_use]
    pub fn supports(self, setting: Setting) -> bool {
        self.settings.contains(&setting)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Model {
    pub pid: u16,
    pub name: &'static str,
    pub support: Support,
    pub capabilities: Capabilities,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelLookup {
    Supported(Model),
    Bootloader,
    Unknown,
}

impl Model {
    pub const SUPPORTED: [Self; 4] = [
        Self {
            pid: 0x2232,
            name: "Arctis Nova 5",
            support: Support::Verified,
            capabilities: Capabilities::ALL_SETTINGS,
        },
        Self {
            pid: 0x2253,
            name: "Arctis Nova 5X",
            support: Support::Verified,
            capabilities: Capabilities::ALL_SETTINGS,
        },
        Self {
            pid: 0x2255,
            name: "Arctis Nova 5X",
            support: Support::Untested,
            capabilities: Capabilities::ALL_SETTINGS,
        },
        Self {
            pid: 0x2264,
            name: "Arctis Nova 5X",
            support: Support::Untested,
            capabilities: Capabilities::ALL_SETTINGS,
        },
    ];

    #[must_use]
    pub fn lookup(pid: u16) -> ModelLookup {
        if BOOTLOADER_PIDS.contains(&pid) {
            return ModelLookup::Bootloader;
        }
        Self::SUPPORTED
            .into_iter()
            .find(|model| model.pid == pid)
            .map_or(ModelLookup::Unknown, ModelLookup::Supported)
    }
}
