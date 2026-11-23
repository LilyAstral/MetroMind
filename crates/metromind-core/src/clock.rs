//! Time source. All energy math takes time from a [`Clock`] so tests can move
//! time forward by hand instead of sleeping.

use chrono::{DateTime, Duration, Utc};
use std::cell::Cell;

/// The only seam for "now". The app uses [`SystemClock`]; tests use [`FakeClock`].
pub trait Clock {
    fn now(&self) -> DateTime<Utc>;
}

/// Wall-clock time in UTC.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// A clock that only moves when told to. Public (not test-only) so integration
/// tests and downstream crates can use it.
#[derive(Debug)]
pub struct FakeClock {
    now: Cell<DateTime<Utc>>,
}

impl FakeClock {
    pub fn new(start: DateTime<Utc>) -> Self {
        Self {
            now: Cell::new(start),
        }
    }

    pub fn advance(&self, by: Duration) {
        self.now.set(self.now.get() + by);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> DateTime<Utc> {
        self.now.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_clock_advances_only_when_told() {
        let start = DateTime::<Utc>::UNIX_EPOCH;
        let clock = FakeClock::new(start);
        assert_eq!(clock.now(), start);
        clock.advance(Duration::hours(5));
        assert_eq!(clock.now(), start + Duration::hours(5));
    }
}
