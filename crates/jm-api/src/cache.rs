use std::path::PathBuf;
use std::time::SystemTime;

/// File-based API response cache.
#[derive(Debug, Clone)]
pub struct ApiCache {
    cache_dir: PathBuf,
}

impl ApiCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Get a cached response if it exists and is not expired.
    pub fn get(&self, key: &str, ttl_secs: u64) -> Option<String> {
        let path = self.path_for(key);
        if !path.exists() {
            return None;
        }

        // Check if the cache entry is still valid
        let metadata = std::fs::metadata(&path).ok()?;
        let modified = metadata.modified().ok()?;
        let age = SystemTime::now().duration_since(modified).ok()?;

        if ttl_secs == 0 || age.as_secs() > ttl_secs {
            return None;
        }

        std::fs::read_to_string(&path).ok()
    }

    /// Store a response in the cache.
    pub fn put(&self, key: &str, data: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.cache_dir)?;
        let path = self.path_for(key);
        std::fs::write(&path, data)
    }

    /// Remove all cached entries.
    pub fn clear(&self) -> std::io::Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    fn path_for(&self, key: &str) -> PathBuf {
        // Sanitize key for use as filename
        let safe_key: String = key
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '.' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.cache_dir.join(format!("{}.json", safe_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ApiCache::new(dir.path().to_path_buf());

        assert!(cache.get("test_key", 3600).is_none());

        cache.put("test_key", r#"{"data": "hello"}"#).unwrap();
        let val = cache.get("test_key", 3600).unwrap();
        assert_eq!(val, r#"{"data": "hello"}"#);
    }

    #[test]
    fn cache_expired() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ApiCache::new(dir.path().to_path_buf());

        cache.put("test_key", "data").unwrap();
        // TTL of 0 means immediately expired
        assert!(cache.get("test_key", 0).is_none());
    }

    #[test]
    fn cache_clear() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ApiCache::new(dir.path().to_path_buf());

        cache.put("key1", "data1").unwrap();
        cache.put("key2", "data2").unwrap();
        cache.clear().unwrap();
        assert!(cache.get("key1", 3600).is_none());
        assert!(cache.get("key2", 3600).is_none());
    }
}
