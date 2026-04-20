use std::time::{Duration, Instant, SystemTime};

/// Monotonic clock.
pub trait Clock {
    fn now(&self) -> Instant;
}

pub struct MonotonicClock;

impl Clock for MonotonicClock {
    #[inline]
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Wall clock (real-time).
pub trait Time {
    fn timestamp(&self) -> Duration;
}

pub struct Real;

impl Time for Real {
    fn timestamp(&self) -> Duration {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
    }
}

#[must_use]
pub fn timestamp() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
}

#[cfg(test)]
pub mod tests {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use super::*;

    pub struct MockTime {
        pub value: Duration,
    }

    impl Time for MockTime {
        fn timestamp(&self) -> Duration {
            self.value
        }
    }

    impl<T> Time for RefCell<T>
    where
        T: Time,
    {
        fn timestamp(&self) -> Duration {
            self.borrow().timestamp()
        }
    }

    impl<T> Time for Rc<T>
    where
        T: Time,
    {
        fn timestamp(&self) -> Duration {
            self.as_ref().timestamp()
        }
    }

    pub struct MockClock(pub Rc<Cell<Instant>>);

    impl Clock for MockClock {
        fn now(&self) -> Instant {
            self.0.get()
        }
    }
}
