use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::SystemTime;

pub struct FileMemoizeThreadSafe<U> {
    cache: RwLock<HashMap<PathBuf, (SystemTime, U)>>,
    max_size: usize,
}

impl<U: Clone + Send + Sync> FileMemoizeThreadSafe<U> {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
        }
    }

    pub fn read<F>(&self, path: &str, mut loader: F) -> std::io::Result<U>
    where
        F: FnMut(&Path) -> std::io::Result<U>,
    {
        let path_buf = PathBuf::from(path);
        
        let metadata = fs::metadata(&path_buf)?;
        let mtime = metadata.modified()?;

        {
            let read_guard = self.cache.read().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::Other, "Lock poisoned")
            })?;
            
            if let Some((cached_time, data)) = read_guard.get(&path_buf) {
                if *cached_time == mtime {
                    return Ok(data.clone());
                }
            }
        }

        let data = loader(&path_buf)?;

        let mut write_guard = self.cache.write().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Lock poisoned")
        })?;

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
