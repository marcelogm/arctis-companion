#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Millis(pub u64);

impl Millis {
    #[must_use]
    pub const fn after(self, duration: u64) -> Self {
        Self(self.0.saturating_add(duration))
    }

    #[must_use]
    pub const fn since(self, earlier: Self) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Millis;

    #[test]
    fn after_adds_a_duration() {
        assert_eq!(Millis(1_000).after(250), Millis(1_250));
    }

    #[test]
    fn after_saturates_at_the_end_of_time() {
        assert_eq!(Millis(u64::MAX).after(1), Millis(u64::MAX));
    }

    #[test]
    fn since_measures_elapsed_time() {
        assert_eq!(Millis(1_250).since(Millis(1_000)), 250);
    }

    #[test]
    fn since_is_zero_when_the_other_instant_is_later() {
        assert_eq!(Millis(1_000).since(Millis(1_250)), 0);
    }

    #[test]
    fn instants_are_ordered() {
        assert!(Millis(1) < Millis(2));
    }
}
