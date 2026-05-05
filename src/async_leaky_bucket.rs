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