use std::sync::{Arc, Mutex};
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