use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;

pub struct AsyncDebounce<T> {
    tx: mpsc::UnboundedSender<T>,
}

impl<T: Send + 'static> AsyncDebounce<T> {
    pub fn new<F, Fut>(wait: Duration, mut logic: F, distinct: bool) -> Self
    where
        T: Send + PartialEq + Clone + 'static, 
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let (tx, mut rx) = mpsc::unbounded_channel::<T>();
    
        tokio::spawn(async move {
            let mut last_pending: Option<T> = None;
            let mut last_executed: Option<T> = None; 
            
            loop {
                let timeout = tokio::time::sleep(wait);
                tokio::pin!(timeout);
    
                tokio::select! {
                    Some(item) = rx.recv() => {
                        last_pending = Some(item);
                    }
                    _ = &mut timeout => {
                        if let Some(item) = last_pending.take() {

                            if distinct && Some(&item) == last_executed.as_ref() {
                                continue; 
                            }
                            
                            last_executed = Some(item.clone());
                            logic(item).await;
                        }
                    }
                    else => break,
                }
            }
        });
    
        Self { tx }
    }

    pub fn send(&self, item: T) {
        let _ = self.tx.send(item);
    }
}