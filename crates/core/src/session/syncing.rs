use crate::protocol::firmware::Firmware;
use crate::protocol::reply::{AudioReadback, UxReadback, observed_settings};
use crate::protocol::{Query, Reply};
use crate::settings::headset_settings::HeadsetSettings;
use crate::settings::values::WirelessMode;

const READS: [Query; 4] = [Query::Audio, Query::Ux, Query::Wireless, Query::Firmware];

#[derive(Clone, Debug, Default)]
pub(crate) struct Readings {
    asked: usize,
    audio: Option<AudioReadback>,
    ux: Option<UxReadback>,
    wireless: Option<WirelessMode>,
    firmware: Option<Firmware>,
}

impl Readings {
    pub(crate) fn next_query(&mut self) -> Option<Query> {
        let query = READS.get(self.asked).copied()?;
        self.asked += 1;
        Some(query)
    }

    pub(crate) const fn wants_query(&self) -> bool {
        self.asked < READS.len()
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.audio.is_some()
            && self.ux.is_some()
            && self.wireless.is_some()
            && self.firmware.is_some()
    }

    pub(crate) fn absorb(&mut self, reply: Reply) {
        match reply {
            Reply::Audio(audio) => self.audio = Some(audio),
            Reply::Ux(ux) => self.ux = Some(ux),
            Reply::Wireless(mode) => self.wireless = Some(mode),
            Reply::Firmware(firmware) => self.firmware = Some(firmware),
            Reply::Status(_) => {}
        }
    }

    pub(crate) fn finish(&self) -> Option<(HeadsetSettings, Firmware)> {
        let settings = observed_settings(self.audio.clone()?, self.ux?, self.wireless?);
        Some((settings, self.firmware?))
    }
}
