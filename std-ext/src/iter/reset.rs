use std::time::Duration;

use crate::time::Time;

pub trait ResetExt: Iterator
where
    Self: Clone + Sized,
{
    fn reset_after<T>(self, get_timestamp: T, reset_delay: Duration) -> Reset<Self, T>
    where
        T: Time,
    {
        Reset::new(self, get_timestamp, reset_delay)
    }
}

impl<I> ResetExt for I where I: Clone + Iterator {}

pub struct Reset<I, T> {
    inner: I,
    initial_inner: I,

    time: T,
    reset_delay: Duration,

    reset_at: Duration,
}

impl<I, T> Reset<I, T>
where
    I: Clone,
    T: Time,
{
    pub fn new(inner: I, time: T, reset_delay: Duration) -> Self {
        Self {
            initial_inner: inner.clone(),
            reset_at: time.timestamp() + reset_delay,

            inner,
            time,
            reset_delay,
        }
    }
}

impl<I, T> Iterator for Reset<I, T>
where
    I: Iterator + Clone,
    T: Time,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let timestamp = self.time.timestamp();
        if timestamp >= self.reset_at {
            self.inner = self.initial_inner.clone();
        }

        self.reset_at = timestamp + self.reset_delay;
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;
    use crate::time::tests::MockTime;

    #[test]
    fn next_resets() {
        let mock_time = Rc::new(RefCell::new(MockTime {
            value: Duration::ZERO,
        }));
        let reset_delay = Duration::from_nanos(2);
        let mut iter = (0..i32::MAX).reset_after(mock_time.clone(), reset_delay);

        assert_eq!(iter.next(), Some(0));
        // Should reset because delay was 2. Sets reset_at to 4.
        mock_time.borrow_mut().value = Duration::from_nanos(2);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        // Should push reset_at to 5.
        mock_time.borrow_mut().value = Duration::from_nanos(3);
        assert_eq!(iter.next(), Some(2));
        // Should not reset yet. Pushes reset_at to 6.
        mock_time.borrow_mut().value = Duration::from_nanos(4);
        assert_eq!(iter.next(), Some(3));
        // Should reset.
        mock_time.borrow_mut().value = Duration::from_nanos(6);
        assert_eq!(iter.next(), Some(0));
    }
}
