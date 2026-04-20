/// Returns an exponential iterator.
///
/// I.e yields 1, 2, 4, 8, 16 ...
#[must_use]
pub const fn exponential() -> Exponential {
    Exponential { current: Some(1) }
}

#[derive(Clone, Debug)]
pub struct Exponential {
    current: Option<u64>,
}

impl Iterator for Exponential {
    type Item = u64;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current?;
        self.current = result.checked_mul(2);
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
        // Iterate until the last value before overflow.
        for _ in 0..63 {
            iter.next();
        }
        // 2^63 (last value that fits in u64).
        assert_eq!(iter.next(), Some(9_223_372_036_854_775_808));
        // 2^64 would overflow, so return None.
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}
