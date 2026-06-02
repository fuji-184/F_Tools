use std::time::Duration;
use tokio::sync::mpsc;

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

ftest::test!(async_debounce_tests, {
    test_debounce_and_distinct.tokio {
        let (tx, mut rx) = mpsc::unbounded_channel::<i32>();
        let debounce = AsyncDebounce::new(
            Duration::from_millis(50),
            move |item| {
                let tx = tx.clone();
                async move {
                    let _ = tx.send(item);
                }
            },
            true,
        );

        debounce.send(1);
        tokio::time::sleep(Duration::from_millis(10)).await;
        debounce.send(2);
        tokio::time::sleep(Duration::from_millis(10)).await;
        debounce.send(3);

        tokio::time::sleep(Duration::from_millis(80)).await;
        assert_eq!(rx.try_recv().unwrap(), 3);
        assert!(rx.try_recv().is_err());

        debounce.send(3);
        tokio::time::sleep(Duration::from_millis(80)).await;
        assert!(rx.try_recv().is_err());

        debounce.send(4);
        tokio::time::sleep(Duration::from_millis(80)).await;
        assert_eq!(rx.try_recv().unwrap(), 4);
        assert!(rx.try_recv().is_err());
    }

    test_debounce_without_distinct.tokio {
        let (tx, mut rx) = mpsc::unbounded_channel::<i32>();
        let debounce = AsyncDebounce::new(
            Duration::from_millis(30),
            move |item| {
                let tx = tx.clone();
                async move {
                    let _ = tx.send(item);
                }
            },
            false,
        );

        debounce.send(42);
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(rx.try_recv().unwrap(), 42);

        debounce.send(42);
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(rx.try_recv().unwrap(), 42);
    }
});