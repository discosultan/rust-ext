/// Returns an exponential iterator.
///
/// I.e yields 1, 2, 4, 8, 16 ...
#[must_use]
pub const fn exponential() -> Exponential {
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
        self.current = self.current.checked_mul(2)?;
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
        for _ in 0..62 {
            iter.next();
        }
        // 2^62 (last value that can be returned without overflow).
        assert_eq!(iter.next(), Some(4_611_686_018_427_387_904));
        // 2^63 would be next, but 2^63 * 2 overflows, so return None.
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}
