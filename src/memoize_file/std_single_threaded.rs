use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct StdFileMemoize<U> {
    cache: HashMap<PathBuf, (SystemTime, U)>,
    max_size: usize,
}

impl<U: Clone> StdFileMemoize<U> {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(max_size),
            max_size,
        }
    }

    pub fn read<F>(&mut self, path: &str, mut loader: F) -> std::io::Result<U>
    where
        F: FnMut(&Path) -> std::io::Result<U>,
    {
        let path_buf = PathBuf::from(path);
        let metadata = fs::metadata(&path_buf)?;
        let mtime = metadata.modified()?;

        if let Some((cached_time, data)) = self.cache.get(&path_buf) {
            if *cached_time == mtime {
                return Ok(data.clone());
            }
        }

        let data = loader(&path_buf)?;

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
