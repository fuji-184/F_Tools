use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::future::Future;

pub struct AsyncFileMemoizeThreadLocal<U> {
    cache: HashMap<PathBuf, (SystemTime, U)>,
    max_size: usize,
}

impl<U: Clone + 'static> AsyncFileMemoizeThreadLocal<U> {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(max_size),
            max_size,
        }
    }

    pub async fn read<F, Fut>(&mut self, path: &str, mut loader: F) -> tokio::io::Result<U>
    where
        F: FnMut(PathBuf) -> Fut,
        Fut: Future<Output = tokio::io::Result<U>>,
    {
        let path_buf = PathBuf::from(path);
        let metadata = tokio::fs::metadata(&path_buf).await?;
        let mtime = metadata.modified()?;

        if let Some((cached_time, data)) = self.cache.get(&path_buf) {
            if *cached_time == mtime {
                return Ok(data.clone());
            }
        }

        let data = loader(path_buf.clone()).await?;

        if self.cache.len() >= self.max_size {
            let key = self.cache.keys().next().cloned();
            if let Some(k) = key {
                self.cache.remove(&k);
            }
        }

        self.cache.insert(path_buf, (mtime, data.clone()));
        Ok(data)
    }
}

#[macro_export]
macro_rules! async_thread_file_memo {
    ($name:ident, $max_size:expr, $u:ty) => {
        use std::borrow::BorrowMut;
        thread_local! {
            static $name: std::cell::RefCell<$crate::memoize_file::AsyncFileMemoizeThreadLocal<$u>> = 
                std::cell::RefCell::new($crate::memoize_file::AsyncFileMemoizeThreadLocal::new($max_size));
        }
    };
}