use std::{fmt::Display, io, str::FromStr};

#[derive(Clone, Debug)]
pub enum RuntimeFlavor {
    #[cfg(feature = "rt")]
    CurrentThread,
    #[cfg(feature = "rt-multi-thread")]
    MultiThread,
}

impl RuntimeFlavor {
    pub fn build(self) -> io::Result<tokio::runtime::Runtime> {
        match self {
            #[cfg(feature = "rt")]
            Self::CurrentThread => tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build(),
            #[cfg(feature = "rt-multi-thread")]
            Self::MultiThread => tokio::runtime::Builder::new_multi_thread()
                .worker_threads(
                    std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
                )
                .enable_all()
                .build(),
        }
    }
}

impl Display for RuntimeFlavor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                #[cfg(feature = "rt")]
                RuntimeFlavor::CurrentThread => "current_thread",
                #[cfg(feature = "rt-multi-thread")]
                RuntimeFlavor::MultiThread => "multi_thread",
            },
        )
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseRuntimeFlavorError {
    #[error("Failed to parse runtime flavor.")]
    Parse,
}

impl FromStr for RuntimeFlavor {
    type Err = ParseRuntimeFlavorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "rt")]
            "current_thread" => Ok(Self::CurrentThread),
            #[cfg(feature = "rt-multi-thread")]
            "multi_thread" => Ok(Self::MultiThread),
            _ => Err(Self::Err::Parse),
        }
    }
}

impl From<RuntimeFlavor> for tokio::runtime::RuntimeFlavor {
    fn from(value: RuntimeFlavor) -> Self {
        match value {
            #[cfg(feature = "rt")]
            RuntimeFlavor::CurrentThread => Self::CurrentThread,
            #[cfg(feature = "rt-multi-thread")]
            RuntimeFlavor::MultiThread => Self::MultiThread,
        }
    }
}
