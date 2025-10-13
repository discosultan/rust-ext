#[cfg(any(feature = "rt", feature = "rt-multi-thread"))]
mod runtime_flavor;
#[cfg(feature = "sync")]
pub mod sync;
pub mod task;

#[cfg(any(feature = "rt", feature = "rt-multi-thread"))]
pub use self::runtime_flavor::*;
