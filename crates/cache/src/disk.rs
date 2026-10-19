use iris_core::{IrisError, IrisResult};
use std::fs;
use std::path::{Path, PathBuf};

pub struct DiskCache {
    cache_dir: PathBuf,
}

impl DiskCache {
    pub fn new(cache_dir: impl AsRef<Path>) -> IrisResult<Self> {
        let dir = cache_dir.as_ref().to_path_buf();
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| IrisError::IoError(e.to_string()))?;
        }
        Ok(Self { cache_dir: dir })
    }

    pub fn get_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.cache", key))
    }

    pub fn read(&self, key: &str) -> Option<Vec<u8>> {
        let path = self.get_path(key);
        fs::read(path).ok()
    }

    pub fn write(&self, key: &str, data: &[u8]) -> IrisResult<()> {
        let path = self.get_path(key);
        fs::write(path, data).map_err(|e| IrisError::IoError(e.to_string()))?;
        Ok(())
    }

    pub fn purge(&self) -> IrisResult<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir).map_err(|e| IrisError::IoError(e.to_string()))?;
            fs::create_dir_all(&self.cache_dir).map_err(|e| IrisError::IoError(e.to_string()))?;
        }
        Ok(())
    }
}
