use std::time::{Duration, Instant};

use crate::time::Clock;

pub trait ResetExt: Iterator
where
    Self: Clone + Sized,
{
    fn reset_after<C>(self, clock: C, delay: Duration) -> Reset<Self, C>
    where
        C: Clock,
    {
        Reset::new(self, clock, delay)
    }
}

impl<I> ResetExt for I where I: Clone + Iterator {}

pub struct Reset<I, C> {
    inner: I,
    initial_inner: I,

    clock: C,
    delay: Duration,

    deadline: Instant,
}

impl<I, C> Reset<I, C>
where
    I: Clone,
    C: Clock,
{
    pub fn new(inner: I, clock: C, delay: Duration) -> Self {
        let now = clock.now();
        Self {
            initial_inner: inner.clone(),
            deadline: now + delay,

            inner,
            clock,
            delay,
        }
    }
}

impl<I, C> Iterator for Reset<I, C>
where
    I: Iterator + Clone,
    C: Clock,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let now = self.clock.now();
        if now >= self.deadline {
            self.inner = self.initial_inner.clone();
        }

        self.deadline = now + self.delay;
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::*;
    use crate::time::tests::MockClock;

    #[test]
    fn next_resets() {
        let start = Instant::now();
        let cell = Rc::new(Cell::new(start));
        let delay = Duration::from_nanos(2);
        let mut iter = Reset::new(0..i32::MAX, MockClock(cell.clone()), delay);

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
        assert_eq!(iter.next(), Some(3));
        // Should reset.
        cell.set(start + Duration::from_nanos(6));
        assert_eq!(iter.next(), Some(0));
    }
}
