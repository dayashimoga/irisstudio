use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryBudgetTier {
    LowMemoryMobile, // 256 MB
    DesktopStandard, // 512 MB
    WorkstationPro,  // 2048 MB
}

impl MemoryBudgetTier {
    pub fn max_bytes(self) -> usize {
        match self {
            Self::LowMemoryMobile => 256 * 1024 * 1024,
            Self::DesktopStandard => 512 * 1024 * 1024,
            Self::WorkstationPro => 2048 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub asset_id: String,
    pub timestamp_num: i64,
    pub timestamp_den: u64,
    pub resolution_tier: u8, // 0 = full, 1 = half, 2 = quarter
}

struct CacheEntry {
    key: CacheKey,
    byte_size: usize,
    data: Vec<u8>,
}

pub struct CacheManager {
    max_bytes: usize,
    current_bytes: usize,
    entries: VecDeque<CacheEntry>,
}

impl CacheManager {
    pub fn new(tier: MemoryBudgetTier) -> Self {
        Self {
            max_bytes: tier.max_bytes(),
            current_bytes: 0,
            entries: VecDeque::new(),
        }
    }

    pub fn with_custom_budget(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            current_bytes: 0,
            entries: VecDeque::new(),
        }
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<Vec<u8>> {
        if let Some(idx) = self.entries.iter().position(|e| &e.key == key) {
            // Move to back (most recently used)
            let entry = self.entries.remove(idx).unwrap();
            let data = entry.data.clone();
            self.entries.push_back(entry);
            Some(data)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: CacheKey, data: Vec<u8>) {
        let byte_size = data.len();

        // If single item exceeds entire cache budget, don't cache it
        if byte_size > self.max_bytes {
            return;
        }

        // Evict LRU items from front until space is available
        while self.current_bytes + byte_size > self.max_bytes && !self.entries.is_empty() {
            let evicted = self.entries.pop_front().unwrap();
            self.current_bytes = self.current_bytes.saturating_sub(evicted.byte_size);
        }

        self.current_bytes += byte_size;
        self.entries.push_back(CacheEntry {
            key,
            byte_size,
            data,
        });
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_bytes = 0;
    }

    pub fn current_bytes(&self) -> usize {
        self.current_bytes
    }

    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

pub static GLOBAL_CACHE: Mutex<Option<CacheManager>> = Mutex::new(None);
