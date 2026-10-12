pub const PAYLOAD_LEN: usize = 64;
pub const REPORT_LEN: usize = PAYLOAD_LEN + 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame([u8; REPORT_LEN]);

impl Frame {
    pub(crate) fn new<const A: usize>(id: u8, args: &[u8; A]) -> Self {
        const { assert!(A < PAYLOAD_LEN) };
        let mut bytes = [0; REPORT_LEN];
        bytes[1] = id;
        bytes[2..2 + A].copy_from_slice(args);
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

pub(crate) fn expect_frame<const N: usize>(bytes: &[u8], id: u8) -> Result<&[u8; N], DecodeError> {
    let frame = bytes.first_chunk::<N>().ok_or(DecodeError::TooShort {
        needed: N,
        got: bytes.len(),
    })?;
    match frame.first() {
        Some(&got) if got != id => Err(DecodeError::UnexpectedId { expected: id, got }),
        _ => Ok(frame),
    }
}

pub(crate) fn byte<const N: usize>(frame: &[u8; N], offset: usize) -> Result<u8, DecodeError> {
    frame.get(offset).copied().ok_or(DecodeError::TooShort {
        needed: offset + 1,
        got: N,
    })
}

pub(crate) fn field<T: TryFrom<u8>, const N: usize>(
    frame: &[u8; N],
    offset: usize,
) -> Result<T, DecodeError> {
    T::try_from(byte(frame, offset)?).map_err(|_| DecodeError::InvalidValue { offset })
}

pub(crate) const fn flag(raw: u8) -> Option<bool> {
    match raw {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

pub(crate) fn flag_field<const N: usize>(
    frame: &[u8; N],
    offset: usize,
) -> Result<bool, DecodeError> {
    flag(byte(frame, offset)?).ok_or(DecodeError::InvalidValue { offset })
}
