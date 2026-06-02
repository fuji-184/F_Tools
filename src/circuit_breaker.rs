
/*
Fault tolerance mechanism to prevent cascading failures in distributed systems.

This structure protects the application from broken dependencies, slow external APIs, 
or unresponsive services. When a targeted downstream dependency fails repeatedly, 
the mechanism automatically cuts off further traffic to that service, failing fast to 
save system resources and allowing the struggling remote system time to recover before 
gradually letting new requests pass through again.
*/

use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
pub enum State { Closed, Open, HalfOpen }

pub struct CircuitBreaker {
    threshold: u32,
    timeout: Duration,
    failures: Mutex<u32>,
    last_failure: Mutex<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            threshold,
            timeout,
            failures: Mutex::new(0),
            last_failure: Mutex::new(None),
        }
    }

    pub fn call<F, T, E>(&self, func: F) -> Result<T, String>
    where F: FnOnce() -> Result<T, E> 
    {
        let mut failures = self.failures.lock().unwrap();
        let mut last_failure = self.last_failure.lock().unwrap();

        if *failures >= self.threshold {
            if let Some(last) = *last_failure {
                if last.elapsed() < self.timeout {
                    return Err("Circuit is OPEN".to_string());
                } else {
                    // half open
                }
            }
        }

        match func() {
            Ok(val) => {
                *failures = 0;
                Ok(val)
            }
            Err(_) => {
                *failures += 1;
                *last_failure = Some(Instant::now());
                Err("Call Failed".to_string())
            }
        }
    }
}

ftest::test!(circuit_breaker_tests, {
    test_closed_state_success {
        let cb = CircuitBreaker::new(3, Duration::from_millis(50));
        let res: Result<i32, String> = cb.call(|| Ok::<i32, &str>(42));
        assert_eq!(res, Ok(42));
    }

    test_trip_to_open_state {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));

        let _ = cb.call(|| Err::<(), &str>("fail"));
        let _ = cb.call(|| Err::<(), &str>("fail"));

        let res = cb.call(|| Ok::<i32, &str>(42));
        assert_eq!(res, Err("Circuit is OPEN".to_string()));
    }

    test_half_open_recovery {
        let cb = CircuitBreaker::new(1, Duration::from_millis(10));

        let _ = cb.call(|| Err::<(), &str>("fail"));

        std::thread::sleep(Duration::from_millis(15));

        let res = cb.call(|| Ok::<i32, &str>(100));
        assert_eq!(res, Ok(100));

        let res_subsequent = cb.call(|| Ok::<i32, &str>(200));
        assert_eq!(res_subsequent, Ok(200));
    }
});