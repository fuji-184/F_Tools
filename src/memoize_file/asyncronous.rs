use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;

pub struct AsyncFileMemoize<U> {
    cache: Arc<RwLock<HashMap<PathBuf, (SystemTime, U)>>>,
    max_size: usize,
}

impl<U: Clone + Send + Sync + 'static> AsyncFileMemoize<U> {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::with_capacity(max_size))),
            max_size,
        }
    }

    pub async fn read<F, Fut>(&self, path: &str, mut loader: F) -> tokio::io::Result<U>
    where
        F: FnMut(PathBuf) -> Fut,
        Fut: std::future::Future<Output = tokio::io::Result<U>>,
    {
        let path_buf = PathBuf::from(path);
        
        let metadata = fs::metadata(&path_buf).await?;
        let mtime = metadata.modified()?;

        {
            let read_guard = self.cache.read().await;
            if let Some((cached_time, data)) = read_guard.get(&path_buf) {
                if *cached_time == mtime {
                    return Ok(data.clone());
                }
            }
        }

        let data = loader(path_buf.clone()).await?;

        let mut write_guard = self.cache.write().await;
        
        if write_guard.len() >= self.max_size {
            let key_to_remove = write_guard.keys().next().cloned();
            if let Some(k) = key_to_remove {
                write_guard.remove(&k);
            }
        }

        write_guard.insert(path_buf, (mtime, data.clone()));
        Ok(data)
    }
}