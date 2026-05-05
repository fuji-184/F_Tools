use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct HeartbeatMonitor {
    workers: Mutex<HashMap<u16, Instant>>, 
    timeout: Duration,
}

impl HeartbeatMonitor {
    pub fn beat(&self, worker_id: u16) {
        self.workers.lock().unwrap().insert(worker_id, Instant::now());
    }

    pub fn get_dead_workers(&self) -> Vec<u16> {
        let now = Instant::now();
        let workers = self.workers.lock().unwrap();
        workers.iter()
            .filter(|&(_, &last_seen)| now.duration_since(last_seen) > self.timeout)
            .map(|(&id, _)| id)
            .collect()
    }
}