use std::{
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

const MIN_INTERVAL_MS: u64 = 1000;

/// Returns `true` at most once per [`MIN_INTERVAL_MS`] for the given state.
///
/// `next_log_ms` holds the earliest time (in milliseconds since a process-wide
/// epoch) at which the next log is allowed; it should start at 0 so that the
/// first call logs immediately.
pub(crate) fn should_log(next_log_ms: &AtomicU64) -> bool {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    let epoch = *EPOCH.get_or_init(Instant::now);
    let now_ms = u64::try_from(epoch.elapsed().as_millis()).unwrap_or(u64::MAX);

    let next = next_log_ms.load(Ordering::Relaxed);
    now_ms >= next
        && next_log_ms
            .compare_exchange(
                next,
                now_ms.saturating_add(MIN_INTERVAL_MS),
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_log_true_once_within_interval() {
        let state = AtomicU64::new(0);
        assert!(should_log(&state));
        assert!(!should_log(&state));
        assert!(!should_log(&state));
    }

    #[test]
    fn should_log_true_again_after_interval() {
        let state = AtomicU64::new(0);
        assert!(should_log(&state));
        // Rewind the next-allowed time to simulate the interval elapsing.
        state.store(0, Ordering::Relaxed);
        assert!(should_log(&state));
    }
}
