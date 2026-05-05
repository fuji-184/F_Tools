use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AtomicTimeSeriesId {
    state: AtomicU64,
    worker_id: u64,
}

impl AtomicTimeSeriesId {
    pub fn new(worker_id: u16) -> Self {
        let initial_ts = Self::timestamp();
        let worker_bits = (worker_id as u64 & 0x3FF) << 12;
        let initial_state = (initial_ts << 22) | worker_bits;
        
        Self {
            state: AtomicU64::new(initial_state),
            worker_id: worker_bits,
        }
    }

    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64
    }

    pub fn next_id(&self) -> u64 {
        loop {
            let current_state = self.state.load(Ordering::Acquire);
            let now = Self::timestamp();
            
            let last_ts = current_state >> 22;
            let last_seq = current_state & 0xFFF;

            let mut next_ts = now;
            let mut next_seq = 0;

            if now <= last_ts {
                next_ts = last_ts;
                next_seq = last_seq + 1;

                if next_seq > 0xFFF {
                    continue; 
                }
            }

            let next_state = (next_ts << 22) | self.worker_id | next_seq;

            if self.state.compare_exchange(
                current_state,
                next_state,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                return next_state;
            }
        }
    }
}