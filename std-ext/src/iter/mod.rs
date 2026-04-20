mod exponential;
mod reset;
mod saturating;

use std::{
    iter::{Chain, Once},
    time::Duration,
};

pub use self::{exponential::*, reset::*, saturating::*};
use crate::time::{Clock, MonotonicClock};

pub type ZeroThenExponentialWithReset =
    Reset<Saturating<Chain<Once<u64>, Exponential>>, MonotonicClock>;
pub type ExponentialWithReset = Reset<Saturating<Exponential>, MonotonicClock>;

/// Returns an iterator where the first item is 0. The following items are
/// exponential. Saturates on overflow. Resets after specified time since last
/// yield to 0.
///
/// I.e yields 0, 1, 2, 4, 8 ... after reset ... 0, 1, 2, 4, 8 ...
#[must_use]
pub fn zero_then_exponential_with_reset(reset_delay: Duration) -> ZeroThenExponentialWithReset {
    zero_then_exponential_with_reset_inner(MonotonicClock, reset_delay)
}

fn zero_then_exponential_with_reset_inner<C>(
    clock: C,
    reset_delay: Duration,
) -> Reset<Saturating<Chain<Once<u64>, Exponential>>, C>
where
    C: Clock,
{
    std::iter::once(0u64)
        .chain(exponential())
        .saturating()
        .reset_after(clock, reset_delay)
}

/// Returns an iterator where the first item is 1. The following items are
/// exponential. Saturates on overflow. Resets after specified time since last
/// yield to 1.
///
/// I.e yields 1, 2, 4, 8 ... after reset ... 1, 2, 4, 8 ...
#[must_use]
pub fn exponential_with_reset(reset_delay: Duration) -> ExponentialWithReset {
    exponential_with_reset_inner(MonotonicClock, reset_delay)
}

fn exponential_with_reset_inner<C>(
    clock: C,
    reset_delay: Duration,
) -> Reset<Saturating<Exponential>, C>
where
    C: Clock,
{
    exponential().saturating().reset_after(clock, reset_delay)
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc, time::Instant};

    use super::*;
    use crate::time::tests::MockClock;

    fn zero_then_exponential_with_reset_mock(
        clock: MockClock,
        reset_delay: Duration,
    ) -> Reset<Saturating<Chain<Once<u64>, Exponential>>, MockClock> {
        Reset::new(
            std::iter::once(0u64).chain(exponential()).saturating(),
            clock,
            reset_delay,
        )
    }

    fn exponential_with_reset_mock(
        clock: MockClock,
        reset_delay: Duration,
    ) -> Reset<Saturating<Exponential>, MockClock> {
        Reset::new(exponential().saturating(), clock, reset_delay)
    }

    #[test]
    fn test_zero_then_exponential_with_reset() {
        let start = Instant::now();
        let cell = Rc::new(Cell::new(start));
        let reset_delay = Duration::from_nanos(2);
        let mut iter = zero_then_exponential_with_reset_mock(MockClock(cell.clone()), reset_delay);

        assert_eq!(iter.next(), Some(0));
        // Should reset because delay was 2. Sets reset_at to 4.
        cell.set(start + Duration::from_nanos(2));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        // Should push reset_at to 5.
        cell.set(start + Duration::from_nanos(3));
        assert_eq!(iter.next(), Some(2));
        // Should not reset yet. Pushes reset_at to 6.
        cell.set(start + Duration::from_nanos(4));
        assert_eq!(iter.next(), Some(4));
        // Should reset.
        cell.set(start + Duration::from_nanos(6));
        assert_eq!(iter.next(), Some(0));
    }

    #[test]
    fn test_exponential_with_reset() {
        let start = Instant::now();
        let cell = Rc::new(Cell::new(start));
        let reset_delay = Duration::from_nanos(2);
        let mut iter = exponential_with_reset_mock(MockClock(cell.clone()), reset_delay);

        assert_eq!(iter.next(), Some(1));
        // Should reset because delay was 2. Sets reset_at to 4.
        cell.set(start + Duration::from_nanos(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        // Should push reset_at to 5.
        cell.set(start + Duration::from_nanos(3));
        assert_eq!(iter.next(), Some(4));
        // Should not reset yet. Pushes reset_at to 6.
        cell.set(start + Duration::from_nanos(4));
        assert_eq!(iter.next(), Some(8));
        // Should reset.
        cell.set(start + Duration::from_nanos(6));
        assert_eq!(iter.next(), Some(1));
    }
}
