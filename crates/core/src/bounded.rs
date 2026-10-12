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
            type Error = $crate::settings::values::OutOfRange;

            fn try_from(value: $raw) -> Result<Self, Self::Error> {
                if (Self::MIN..=Self::MAX).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err($crate::settings::values::OutOfRange)
                }
            }
        }
    };
}

pub(crate) use bounded;
