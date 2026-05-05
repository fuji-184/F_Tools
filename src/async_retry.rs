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