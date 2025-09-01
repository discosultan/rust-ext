use std::time::{Duration, SystemTime};

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

#[cfg(test)]
pub mod tests {
    use std::{cell::RefCell, rc::Rc};

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
}
