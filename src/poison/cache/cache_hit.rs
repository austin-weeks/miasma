use std::sync::Arc;

use tokio::sync::Semaphore;

/// Returns a function suitable for use in [`crate::poison::PoisonSource`].
///
/// Ensures a hit rate of ~5% with a minumum of 5 in-flight missers.
/// The goal of this formula is so that, on average, cached entries are only served
/// 20 times before being refreshed.
pub fn make_cache_hit_fn(
    semaphore: Arc<Semaphore>,
    max_in_flight: usize,
) -> Box<dyn Fn() -> bool + Send + Sync> {
    Box::new(move || {
        let currently_in_flight = max_in_flight.saturating_sub(semaphore.available_permits());
        if currently_in_flight == 0 {
            return false;
        }

        let chance = hit_probability(currently_in_flight);
        rand::random_bool(chance)
    })
}

fn hit_probability(currently_in_flight: usize) -> f64 {
    #[expect(clippy::cast_precision_loss)]
    let in_flight = currently_in_flight as f64;
    (1.0 - 5.0 / in_flight).clamp(0.0, 0.95)
}

#[cfg(test)]
mod test {
    #![allow(clippy::float_cmp)]
    use super::*;

    #[test]
    fn hit_rate_always_zero_with_five_or_less_in_flight() {
        for in_flight in 0..=5 {
            assert_eq!(hit_probability(in_flight), 0.0);
        }
    }
    #[test]
    fn hit_rate_scales_with_in_flight() {
        let cases = [
            (6, 0.17),
            (10, 0.50),
            (15, 0.67),
            (25, 0.80),
            (50, 0.90),
            (75, 0.93),
            (100, 0.95),
        ];
        for (in_flight, expected) in cases {
            let actual = hit_probability(in_flight);
            assert!((expected - 0.01..=expected + 0.01).contains(&actual));
        }
    }

    #[test]
    fn hit_rate_caps_at_95_percent() {
        let cases = [100, 150, 200, 250, 500, 1000, 10_000];

        for in_flight in cases {
            assert_eq!(hit_probability(in_flight), 0.95);
        }
    }

    #[test]
    fn in_flight_calculated_via_semaphore() {
        // Sem with all permits taken (5 in-flight).
        let sem = Arc::new(Semaphore::new(0));
        let cache_hit_fn = make_cache_hit_fn(sem, 5);
        assert!(!cache_hit_fn());
    }
}
