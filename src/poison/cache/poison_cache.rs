use std::collections::VecDeque;

use bytes::Bytes;

const MAX_ENTRIES: usize = 1024;
const MAX_BYTES: usize = 6 * 1024 * 1024; // 6MB

pub struct PoisonCache {
    queue: VecDeque<Entry>,
    total_bytes: usize,
    max_size: usize,
    max_total_bytes: usize,
}

struct Entry {
    poison: Bytes,
    allocated_size: usize,
}

#[derive(Debug)]
pub struct EntryTooBigError;

impl PoisonCache {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(MAX_ENTRIES),
            total_bytes: 0,
            max_size: MAX_ENTRIES,
            max_total_bytes: MAX_BYTES,
        }
    }

    /// Get a random entry in the cache, or `None` if empty.
    pub fn get_random(&self) -> Option<Bytes> {
        // rand::random_range() panics if the range is empty...
        if self.queue.is_empty() {
            return None;
        }
        let ind = rand::random_range(..self.queue.len());
        self.queue
            .get(ind)
            .map(|Entry { poison, .. }| poison.clone())
    }

    /// Insert a poison entry into the cache, evicting the oldest entries if full.
    /// `allocated_size` should be the actual allocated size of the underlying data, if known.
    /// Returns [`EntryTooBigError`] if the entry exceeds the cache's max total byte size.
    pub fn insert(
        &mut self,
        poison: Bytes,
        allocated_size: Option<usize>,
    ) -> Result<(), EntryTooBigError> {
        let allocated_size = allocated_size.unwrap_or(poison.len());
        if allocated_size > self.max_total_bytes {
            return Err(EntryTooBigError);
        }

        while self.full() || !self.has_room_for(allocated_size) {
            self.evict();
        }
        self.total_bytes += allocated_size;
        self.queue.push_back(Entry {
            poison,
            allocated_size,
        });
        Ok(())
    }

    pub fn max_bytes(&self) -> usize {
        self.max_total_bytes
    }

    fn evict(&mut self) {
        let Some(Entry { allocated_size, .. }) = self.queue.pop_front() else {
            return;
        };
        self.total_bytes = self.total_bytes.saturating_sub(allocated_size);
    }

    fn full(&self) -> bool {
        self.queue.len() >= self.max_size
    }

    fn has_room_for(&self, entry_size: usize) -> bool {
        self.total_bytes + entry_size <= self.max_total_bytes
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let cache = PoisonCache::new();
        assert_eq!(cache.queue.capacity(), MAX_ENTRIES);
        assert_eq!(cache.total_bytes, 0);
        assert_eq!(cache.max_size, MAX_ENTRIES);
        assert_eq!(cache.max_total_bytes, MAX_BYTES);
    }

    #[test]
    fn get_empty_cache() {
        let cache = PoisonCache::new();
        assert!(cache.get_random().is_none());
    }

    #[test]
    fn get_non_empty_cache() {
        let mut cache = PoisonCache::new();
        cache.queue.push_front(Entry {
            poison: Bytes::new(),
            allocated_size: 0,
        });
        assert!(cache.get_random().is_some());
    }

    #[test]
    fn insert_rejects_too_large_entries() {
        // Rejection based on size if provided.
        let mut cache = PoisonCache::new();
        let result = cache.insert(Bytes::new(), Some(MAX_BYTES + 1));
        assert!(result.is_err());

        // Rejection based on Bytes::len() if size not provided.
        let result = cache.insert(Bytes::from(vec![0u8; MAX_BYTES + 1]), None);
        assert!(result.is_err());
    }

    #[test]
    fn insert_evicts_while_cache_full() {
        let mut cache = PoisonCache::new();
        let first_entry = Bytes::from_static("first!".as_bytes());
        cache.queue.push_back(Entry {
            poison: first_entry,
            allocated_size: 0,
        });

        // Fill the rest of the queue
        while cache.queue.len() < MAX_ENTRIES {
            cache.queue.push_back(Entry {
                poison: Bytes::new(),
                allocated_size: 0,
            });
        }

        _ = cache.insert(Bytes::new(), None);
        assert_eq!(cache.queue.len(), MAX_ENTRIES);
        let result = cache
            .queue
            .iter()
            .find(|Entry { poison, .. }| &poison[..] == "first!".as_bytes());
        assert!(result.is_none());
    }

    #[test]
    fn insert_evicts_until_cache_has_room() {
        let mut cache = PoisonCache::new();
        cache.insert(Bytes::new(), Some(MAX_BYTES)).unwrap();

        _ = cache.insert(Bytes::new(), Some(1));
        assert_eq!(cache.total_bytes, 1);
        assert_eq!(cache.queue.len(), 1);
    }

    #[test]
    fn insert_pushes_and_increments_byte_counter() {
        let mut cache = PoisonCache::new();

        let result = cache.insert(Bytes::new(), Some(10));
        assert!(result.is_ok());
        assert_eq!(cache.total_bytes, 10);
        assert!(cache.queue.front().is_some());

        // Byte increment falls back to Bytes::len().
        _ = cache.insert(Bytes::from_static(&[0]), None); // len 1
        assert_eq!(cache.total_bytes, 10 + 1);
    }
}
