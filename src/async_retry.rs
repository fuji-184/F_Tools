
/*
Asynchronous retry mechanism with optional backoff scaling.

This function automatically re evaluates an operation producing a Result future upon failure.
It repeats execution up to a specified maximum attempt threshold before escalating the final 
error. Between retries, it yields control non blockingly using either a static sleep 
duration or an increased backoff delay proportional to the current attempt count.
*/

use std::time::Duration;
use tokio::time::sleep;

pub async fn async_retry<T, E, F, Fut>(
    mut logic: F,
    max_attempts: usize,
    interval: Duration,
    exponential: bool,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut attempt = 0;

    loop {
        match logic().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempt += 1;
                if attempt > max_attempts {
                    return Err(e);
                }

                let delay = if exponential {
                    interval * attempt as u32
                } else {
                    interval
                };

                sleep(delay).await;
            }
        }
    }
}

ftest::test!(async_retry_tests, {
    test_retry_success.tokio {
        let mut attempts = 0;
        let result = async_retry(
            || {
                attempts += 1;
                async move {
                    if attempts < 3 {
                        Err("error")
                    } else {
                        Ok(42)
                    }
                }
            },
            3,
            Duration::from_millis(5),
            false,
        )
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempts, 3);
    }

    test_retry_max_attempts_exceeded.tokio {
        let mut attempts = 0;
        let result: Result<(), &str> = async_retry(
            || {
                attempts += 1;
                async move { Err("persistent error") }
            },
            2,
            Duration::from_millis(5),
            true,
        )
        .await;

        assert_eq!(result, Err("persistent error"));
        assert_eq!(attempts, 3);
    }
});