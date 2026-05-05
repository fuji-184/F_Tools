use tokio::sync::OnceCell;

pub struct AsyncOnce<T> {
    cell: OnceCell<T>,
}

impl<T> AsyncOnce<T> {
    pub fn new() -> Self {
        Self { cell: OnceCell::new() }
    }

    pub async fn get_or_init<F, Fut>(&self, init: F) -> &T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        self.cell.get_or_init(init).await
    }
}