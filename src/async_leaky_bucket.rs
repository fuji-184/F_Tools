
/*
Asynchronous leaky bucket rate limiter for smooth traffic shaping.

This structure controls request distribution by accumulating tokens at a steady refill rate 
up to a maximum burst capacity. When resources are depleted, tasks seeking acquisition are 
suspended non-blockingly via calculated async timeouts until the bucket recovers sufficient 
capacity, enforcing a consistent throughput ceiling across concurrent execution contexts.
*/

use tokio::time::{Duration, Instant, sleep};
use tokio::sync::Mutex;

pub struct AsyncLeakyBucket {
    capacity: f64,
    refill_rate: f64,
    state: Mutex<BucketState>,
}

struct BucketState {
    tokens: f64,
    last_update: Instant,
}

impl AsyncLeakyBucket {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            capacity,
            refill_rate,
            state: Mutex::new(BucketState {
                tokens: capacity,
                last_update: Instant::now(),
            }),
        }
    }

    pub async fn acquire(&self) {
        loop {
            let now = Instant::now();
            let sleep_duration = {
                let mut state = self.state.lock().await;
                
                let elapsed = now.duration_since(state.last_update).as_secs_f64();
                state.tokens = (state.tokens + elapsed * self.refill_rate).min(self.capacity);
                state.last_update = now;

                if state.tokens >= 1.0 {
                    state.tokens -= 1.0;
                    return;
                }

                let needed = 1.0 - state.tokens;
                Duration::from_secs_f64(needed / self.refill_rate)
            };

            sleep(sleep_duration).await;
        }
    }
}

ftest::test!(async_leaky_bucket_tests, {
    test_immediate_acquires.tokio {
        let bucket = AsyncLeakyBucket::new(3.0, 1.0);

        let start = Instant::now();
        bucket.acquire().await;
        bucket.acquire().await;
        bucket.acquire().await;
        
        assert!(start.elapsed() < Duration::from_millis(10));
    }

    test_rate_limiting_and_refill.tokio {
        let bucket = AsyncLeakyBucket::new(1.0, 10.0);

        bucket.acquire().await;

        let start = Instant::now();
        bucket.acquire().await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(90));
        assert!(elapsed < Duration::from_millis(150));
    }
});