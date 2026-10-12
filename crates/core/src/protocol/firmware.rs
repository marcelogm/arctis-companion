use std::fmt;

use super::command::Query;
use super::frame::{DecodeError, expect_frame};

const SLOT_LEN: usize = 12;
const RECEIVER_SLOT: usize = 1;
const HEADSET_SLOT: usize = 25;
const FIRMWARE_LEN: usize = HEADSET_SLOT + SLOT_LEN;
const THREE_PART_DOTS: [usize; 3] = [1, 4, 8];
const THREE_PART_PAIRS: [usize; 3] = [2, 6, 9];
const TWO_PART_PAIRS: [usize; 2] = [7, 10];
const UNREACHABLE: Version = Version {
    major: 0,
    minor: 0,
    patch: 0,
};

type Slot = [u8; SLOT_LEN];

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

pub(crate) fn decode_firmware(bytes: &[u8]) -> Result<Firmware, DecodeError> {
    let frame = expect_frame::<FIRMWARE_LEN>(bytes, Query::Firmware.id())?;
    Ok(Firmware {
        receiver: slot(frame, RECEIVER_SLOT).and_then(decode_slot),
        headset: slot(frame, HEADSET_SLOT).and_then(decode_slot),
    })
}

fn slot(frame: &[u8; FIRMWARE_LEN], start: usize) -> Option<&Slot> {
    frame.get(start..)?.first_chunk()
}

fn decode_slot(slot: &Slot) -> Option<Version> {
    let version = if is_three_part(slot) {
        three_part(slot)
    } else {
        two_part(slot)
    }?;
    (version != UNREACHABLE).then_some(version)
}

fn is_three_part(slot: &Slot) -> bool {
    THREE_PART_DOTS
        .iter()
        .all(|&position| slot.get(position) == Some(&b'.'))
}

fn three_part(slot: &Slot) -> Option<Version> {
    let [Some(major), Some(minor), Some(patch)] =
        THREE_PART_PAIRS.map(|start| digit_pair(slot, start))
    else {
        return None;
    };
    Some(Version {
        major,
        minor,
        patch,
    })
}

fn two_part(slot: &Slot) -> Option<Version> {
    let [Some(major), Some(minor)] = TWO_PART_PAIRS.map(|start| digit_pair(slot, start)) else {
        return None;
    };
    Some(Version {
        major,
        minor,
        patch: 0,
    })
}

fn digit_pair(slot: &Slot, start: usize) -> Option<u8> {
    let high = digit(*slot.get(start)?)?;
    let low = digit(*slot.get(start + 1)?)?;
    Some(high * 16 + low)
}

fn digit(byte: u8) -> Option<u8> {
    byte.checked_sub(b'0').filter(|value| *value < 10)
}
