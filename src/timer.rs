// ---------------------------------------------------------------------
// Gufo Ping: Timer implementation
// ---------------------------------------------------------------------
// Copyright (C) 2022-26, Gufo Labs
// ---------------------------------------------------------------------

use coarsetime::Clock;
use std::time::Instant;

pub(crate) enum Timer {
    Monotonic(Instant),
    MonotonicCoarse,
}

impl Timer {
    // Create new timer
    // @todo: Auto-detect availability?
    pub(crate) fn new(coarse: bool) -> Timer {
        if coarse {
            Timer::MonotonicCoarse
        } else {
            Timer::Monotonic(Instant::now())
        }
    }

    // Get current timestamp in nanoseconds
    pub(crate) fn get_ts(&self) -> u64 {
        match self {
            Timer::Monotonic(start) => start.elapsed().as_nanos() as u64,
            Timer::MonotonicCoarse => Clock::now_since_epoch().as_nanos(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monotonic() {
        let timer = Timer::Monotonic(Instant::now());
        let ts0 = timer.get_ts();
        let ts1 = timer.get_ts();
        assert!(ts0 <= ts1);
        let ts2 = timer.get_ts();
        assert!(ts1 <= ts2);
    }
    #[test]
    fn test_monotonic_coarse() {
        let timer = Timer::MonotonicCoarse;
        let ts0 = timer.get_ts();
        let ts1 = timer.get_ts();
        assert!(ts0 <= ts1);
        let ts2 = timer.get_ts();
        assert!(ts1 <= ts2);
    }
    #[test]
    fn test_new_monotonic() {
        let timer = Timer::new(false);
        let ts0 = timer.get_ts();
        let ts1 = timer.get_ts();
        assert!(ts0 <= ts1);
        let ts2 = timer.get_ts();
        assert!(ts1 <= ts2);
    }
    #[test]
    fn test_new_monotonic_coarse() {
        let timer = Timer::new(true);
        let ts0 = timer.get_ts();
        let ts1 = timer.get_ts();
        assert!(ts0 <= ts1);
        let ts2 = timer.get_ts();
        assert!(ts1 <= ts2);
    }
}
