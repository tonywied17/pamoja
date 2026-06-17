//! Duplicate suppression for flooded packets.

/// A fixed-size memory of the most recently seen packets, so a node relays each one once.
///
/// In a flood every node rebroadcasts what it hears, so the same packet reaches a node
/// from several neighbours. Without a memory of what it has already handled, a node would
/// relay every copy and the flood would multiply without bound. This cache remembers the
/// last `N` packet keys (a [`dedup_key`](crate::Frame::dedup_key), the source and sequence
/// id) in a ring, evicting the oldest as new ones arrive, so the test for "have I seen
/// this?" stays cheap and needs no allocation. `N` sets how far back the memory reaches;
/// a small power of two such as 32 or 64 suits a local mesh.
///
/// # Examples
///
/// ```
/// use pamoja_mesh::SeenCache;
///
/// let mut seen: SeenCache<8> = SeenCache::new();
/// assert!(seen.record((0x42, 1)));  // first time: newly recorded
/// assert!(!seen.record((0x42, 1))); // again: a duplicate
/// assert!(seen.record((0x42, 2)));  // a different packet
/// ```
#[derive(Clone, Copy, Debug)]
pub struct SeenCache<const N: usize> {
    keys: [Option<(u32, u16)>; N],
    next: usize,
}

impl<const N: usize> SeenCache<N> {
    /// Creates an empty cache.
    ///
    /// # Returns
    ///
    /// A cache holding no keys.
    pub const fn new() -> Self {
        SeenCache {
            keys: [None; N],
            next: 0,
        }
    }

    /// Reports whether a key is currently remembered.
    ///
    /// # Arguments
    ///
    /// * `key` - the packet key to look for, from [`dedup_key`](crate::Frame::dedup_key).
    ///
    /// # Returns
    ///
    /// `true` if the key is in the cache.
    pub fn contains(&self, key: (u32, u16)) -> bool {
        self.keys.contains(&Some(key))
    }

    /// Records a key, reporting whether it was new.
    ///
    /// This is the flood test: record the key of a received packet, and act on the packet
    /// only when this returns `true`. The oldest remembered key is evicted once the cache
    /// is full.
    ///
    /// # Arguments
    ///
    /// * `key` - the packet key to record, from [`dedup_key`](crate::Frame::dedup_key).
    ///
    /// # Returns
    ///
    /// `true` if the key was not already remembered (the packet is new), `false` if it was
    /// (the packet is a duplicate).
    pub fn record(&mut self, key: (u32, u16)) -> bool {
        if self.contains(key) {
            return false;
        }
        // A zero-capacity cache remembers nothing, so every key reads as new.
        if N == 0 {
            return true;
        }
        self.keys[self.next] = Some(key);
        self.next = (self.next + 1) % N;
        true
    }
}

impl<const N: usize> Default for SeenCache<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_is_new_once_then_a_duplicate() {
        let mut seen: SeenCache<8> = SeenCache::new();
        assert!(seen.record((1, 1)));
        assert!(!seen.record((1, 1)));
        assert!(seen.contains((1, 1)));
    }

    #[test]
    fn different_sources_and_ids_are_distinct() {
        let mut seen: SeenCache<8> = SeenCache::new();
        assert!(seen.record((1, 1)));
        assert!(seen.record((1, 2)));
        assert!(seen.record((2, 1)));
        assert!(!seen.record((1, 1)));
    }

    #[test]
    fn the_oldest_key_is_evicted_when_full() {
        let mut seen: SeenCache<2> = SeenCache::new();
        assert!(seen.record((0, 1)));
        assert!(seen.record((0, 2)));
        // Recording a third key evicts the oldest, (0, 1).
        assert!(seen.record((0, 3)));
        assert!(!seen.contains((0, 1)));
        assert!(seen.contains((0, 2)));
        assert!(seen.contains((0, 3)));
        // The evicted key is treated as new again.
        assert!(seen.record((0, 1)));
    }

    #[test]
    fn an_empty_cache_remembers_nothing() {
        let seen: SeenCache<4> = SeenCache::default();
        assert!(!seen.contains((1, 1)));
    }

    #[test]
    fn a_zero_capacity_cache_reads_every_key_as_new_without_panicking() {
        let mut seen: SeenCache<0> = SeenCache::new();
        assert!(seen.record((1, 1)));
        assert!(seen.record((1, 1))); // nothing was remembered, so it is new again
        assert!(!seen.contains((1, 1)));
    }
}
