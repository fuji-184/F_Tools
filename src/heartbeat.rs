
/*
Distributed health-check primitive for monitoring the liveness of parallel worker nodes.

This structure tracks the last successful communication timestamp from multiple worker 
instances and identifies those that have exceeded a predefined survival threshold. It is 
primarily used in distributed system coordination—such as managing cluster membership, 
triggering failover routines for crashed microservices, or cleaning up stale job registrations—by 
providing a robust, lock-protected mechanism to differentiate between transient network jitter 
and actual node failure.
*/

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

ftest::test!(heartbeat_monitor_tests, {
    test_beat_and_active_workers {
        let monitor = HeartbeatMonitor {
            workers: Mutex::new(HashMap::new()),
            timeout: Duration::from_millis(50),
        };

        monitor.beat(1);
        monitor.beat(2);

        let dead = monitor.get_dead_workers();
        assert!(dead.is_empty());
    }

    test_detect_dead_workers {
        let monitor = HeartbeatMonitor {
            workers: Mutex::new(HashMap::new()),
            timeout: Duration::from_millis(10),
        };

        monitor.beat(1);
        monitor.beat(2);

        std::thread::sleep(Duration::from_millis(15));
        monitor.beat(2);

        let dead = monitor.get_dead_workers();
        assert_eq!(dead, vec![1]);
    }
});