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