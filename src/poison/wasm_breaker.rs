#[derive(Default)]
pub(super) struct WasmBreaker {
    failures: u8,
    retry_at_ms: f64,
}

impl WasmBreaker {
    pub fn permits(&mut self, now_ms: f64) -> bool {
        if now_ms < self.retry_at_ms {
            return false;
        }
        if self.failures >= 5 {
            // One recovery probe per cooldown, including if the probe is dropped.
            self.retry_at_ms = now_ms + 20_000.0;
        }
        true
    }

    pub fn record(&mut self, failed: bool, now_ms: f64) {
        if failed {
            self.failures = self.failures.saturating_add(1);
            if self.failures >= 5 {
                self.retry_at_ms = now_ms + 20_000.0;
            }
        } else {
            *self = Self::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_after_failures_and_resets_after_success() {
        let mut breaker = WasmBreaker::default();
        for _ in 0..4 {
            breaker.record(true, 100.0);
            assert!(breaker.permits(100.0));
        }
        breaker.record(true, 100.0);
        assert!(!breaker.permits(20_099.0));
        assert!(breaker.permits(20_100.0));
        assert!(!breaker.permits(20_100.0));
        assert!(!breaker.permits(40_099.0));
        assert!(breaker.permits(40_100.0));
        breaker.record(true, 40_100.0);
        assert!(!breaker.permits(40_100.0));
        breaker.record(false, 60_100.0);
        assert!(breaker.permits(60_100.0));
        assert_eq!(breaker.failures, 0);
    }
}
