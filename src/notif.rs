use tokio::sync::watch;

pub struct Notif<T> {
    tx: watch::Sender<T>,
    rx: watch::Receiver<T>,
}

impl<T: Clone> Notif<T> {
    pub fn new(initial: T) -> Self {
        let (tx, rx) = watch::channel(initial);
        Self { tx, rx }
    }

    pub fn set(&self, value: T) {
        let _ = self.tx.send(value);
    }

    pub fn subscribe(&self) -> watch::Receiver<T> {
        self.rx.clone()
    }
}

ftest::test!(notif_tests, {
    test_initial_value {
        let notif = Notif::new(10);
        let rx = notif.subscribe();

        assert_eq!(*rx.borrow(), 10);
    }

    test_set_notifies_subscribers.tokio {
        let notif = Notif::new(10);
        let mut rx = notif.subscribe();

        notif.set(20);

        assert_eq!(*rx.borrow(), 20);
        assert!(rx.changed().await.is_ok());
        assert_eq!(*rx.borrow_and_update(), 20);
    }
});