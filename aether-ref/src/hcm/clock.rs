use std::time::{Duration, Instant};

/// Trait for monotonic clock access, enabling test injection.
pub trait MonotonicClock: Send + Sync {
    fn now(&self) -> Instant;
    fn elapsed_since(&self, earlier: Instant) -> Duration {
        self.now().duration_since(earlier)
    }
}

/// Real monotonic clock using std::time::Instant.
pub struct SystemClock;

impl MonotonicClock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Mock clock for testing — allows manual time advancement.
#[cfg(test)]
pub struct MockClock {
    base: Instant,
    offset: std::sync::Mutex<Duration>,
}

#[cfg(test)]
impl MockClock {
    pub fn new() -> Self {
        Self {
            base: Instant::now(),
            offset: std::sync::Mutex::new(Duration::ZERO),
        }
    }

    pub fn advance(&self, duration: Duration) {
        let mut offset = self.offset.lock().unwrap();
        *offset += duration;
    }
}

#[cfg(test)]
impl MonotonicClock for MockClock {
    fn now(&self) -> Instant {
        self.base + *self.offset.lock().unwrap()
    }
}
