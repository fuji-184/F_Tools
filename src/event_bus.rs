use tokio::sync::broadcast;

pub struct EventBus<T> {
    tx: broadcast::Sender<T>,
}

impl<T: Clone> EventBus<T> {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn publish(&self, event: T) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.tx.subscribe()
    }
}

ftest::test!(event_bus_tests, {
    test_publish_and_receive.tokio {
        let bus = EventBus::new(10);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish(42);

        assert_eq!(rx1.recv().await, Ok(42));
        assert_eq!(rx2.recv().await, Ok(42));
    }

    test_multiple_messages.tokio {
        let bus = EventBus::new(5);
        let mut rx = bus.subscribe();

        bus.publish(100);
        bus.publish(200);

        assert_eq!(rx.recv().await, Ok(100));
        assert_eq!(rx.recv().await, Ok(200));
    }
});