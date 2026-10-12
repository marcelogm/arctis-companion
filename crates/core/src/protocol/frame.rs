pub const PAYLOAD_LEN: usize = 64;
pub const REPORT_LEN: usize = PAYLOAD_LEN + 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame([u8; REPORT_LEN]);

impl Frame {
    pub(crate) fn new(id: u8, args: &[u8]) -> Self {
        let mut bytes = [0; REPORT_LEN];
        bytes[1] = id;
        bytes[2..2 + args.len()].copy_from_slice(args);
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; REPORT_LEN] {
        &self.0
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.0[1..]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    TooShort { needed: usize, got: usize },
    UnexpectedId { expected: u8, got: u8 },
    InvalidValue { offset: usize },
}

pub(crate) fn expect_frame(bytes: &[u8], id: u8, needed: usize) -> Result<&[u8], DecodeError> {
    let frame = bytes.get(..needed).ok_or(DecodeError::TooShort {
        needed,
        got: bytes.len(),
    })?;
    if frame[0] == id {
        Ok(frame)
    } else {
        Err(DecodeError::UnexpectedId {
            expected: id,
            got: frame[0],
        })
    }
}

pub(crate) fn field<T: TryFrom<u8>>(frame: &[u8], offset: usize) -> Result<T, DecodeError> {
    T::try_from(frame[offset]).map_err(|_| DecodeError::InvalidValue { offset })
}

pub(crate) const fn flag(raw: u8) -> Option<bool> {
    match raw {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

pub(crate) fn flag_field(frame: &[u8], offset: usize) -> Result<bool, DecodeError> {
    flag(frame[offset]).ok_or(DecodeError::InvalidValue { offset })
}
