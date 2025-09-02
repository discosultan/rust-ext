/// Returns an exponential iterator.
///
/// I.e yields 1, 2, 4, 8, 16 ...
#[must_use]
pub fn exponential() -> Exponential {
    Exponential { current: 1 }
}

#[derive(Clone, Debug)]
pub struct Exponential {
    current: u64,
}

impl Iterator for Exponential {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current;
        // Check for overflow.
        if self.current > 9_223_372_036_854_775_808 {
            return None;
        }
        self.current = self.current.saturating_mul(2);
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_doubles_every_iteration() {
        let mut iter = exponential();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(8));
    }

    #[test]
    fn next_stops_on_overflow() {
        let mut iter = exponential();
        // Iterate until the last value before overflow excluding.
        for _ in 0..63 {
            iter.next();
        }
        // 2^63
        assert_eq!(iter.next(), Some(9_223_372_036_854_775_808));
        // Next calls should result in overflow and hence return `None`.
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}
