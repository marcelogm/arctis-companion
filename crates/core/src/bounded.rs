#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutOfRange;

macro_rules! bounded {
    ($name:ident, $raw:ty, $min:expr, $max:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name($raw);

        impl $name {
            pub const MIN: $raw = $min;
            pub const MAX: $raw = $max;

            #[must_use]
            pub const fn get(self) -> $raw {
                self.0
            }
        }

        impl TryFrom<$raw> for $name {
            type Error = $crate::bounded::OutOfRange;

            fn try_from(value: $raw) -> Result<Self, Self::Error> {
                if (Self::MIN..=Self::MAX).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err($crate::bounded::OutOfRange)
                }
            }
        }
    };
}

macro_rules! raw_enum {
    ($name:ident { $($variant:ident = $raw:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: [Self; [$($raw),+].len()] = [$(Self::$variant),+];

            #[must_use]
            pub const fn raw(self) -> u8 {
                match self {
                    $(Self::$variant => $raw),+
                }
            }
        }

        impl TryFrom<u8> for $name {
            type Error = $crate::bounded::OutOfRange;

            fn try_from(raw: u8) -> Result<Self, Self::Error> {
                Self::ALL
                    .into_iter()
                    .find(|variant| variant.raw() == raw)
                    .ok_or($crate::bounded::OutOfRange)
            }
        }
    };
}

pub(crate) use {bounded, raw_enum};
