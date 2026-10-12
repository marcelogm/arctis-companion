use std::fmt;

const SLOT_LEN: usize = 12;
const RECEIVER_SLOT: usize = 1;
const HEADSET_SLOT: usize = 25;
pub(crate) const FIRMWARE_LEN: usize = HEADSET_SLOT + SLOT_LEN;
const DOT_POSITIONS: [usize; 3] = [1, 4, 8];
const UNREACHABLE: Version = Version {
    major: 0,
    minor: 0,
    patch: 0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Firmware {
    pub receiver: Option<Version>,
    pub headset: Option<Version>,
}

pub(crate) fn decode_firmware(frame: &[u8]) -> Firmware {
    Firmware {
        receiver: decode_slot(slot(frame, RECEIVER_SLOT)),
        headset: decode_slot(slot(frame, HEADSET_SLOT)),
    }
}

fn slot(frame: &[u8], start: usize) -> &[u8] {
    &frame[start..start + SLOT_LEN]
}

fn decode_slot(slot: &[u8]) -> Option<Version> {
    let version = if is_dotted(slot) {
        dotted(slot)
    } else {
        packed(slot)
    }?;
    (version != UNREACHABLE).then_some(version)
}

fn is_dotted(slot: &[u8]) -> bool {
    DOT_POSITIONS.iter().all(|&position| slot[position] == b'.')
}

fn dotted(slot: &[u8]) -> Option<Version> {
    Some(Version {
        major: digit_pair(slot, 2)?,
        minor: digit_pair(slot, 6)?,
        patch: digit_pair(slot, 9)?,
    })
}

fn packed(slot: &[u8]) -> Option<Version> {
    Some(Version {
        major: digit_pair(slot, 7)?,
        minor: digit_pair(slot, 10)?,
        patch: 0,
    })
}

fn digit_pair(slot: &[u8], start: usize) -> Option<u8> {
    Some(digit(slot[start])? * 16 + digit(slot[start + 1])?)
}

fn digit(byte: u8) -> Option<u8> {
    byte.checked_sub(b'0').filter(|value| *value < 10)
}
