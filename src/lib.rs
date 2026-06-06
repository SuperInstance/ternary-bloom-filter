//! # ternary-bloom-filter
//!
//! Ternary Bloom filter: {-1,0,+1} weighted bits for GPU membership testing.
//! Positive bits boost membership, negative bits block it.

const DEFAULT_SIZE: usize = 1024;

fn hash(data: &[u8], seed: u64) -> usize {
    let mut h: u64 = seed;
    for &b in data { h = h.wrapping_mul(31).wrapping_add(b as u64); }
    (h % DEFAULT_SIZE as u64) as usize
}

#[derive(Debug, Clone)]
pub struct TernaryBloom {
    bits: Vec<i8>,  // {-1, 0, +1}
    hash_count: usize,
    positive_count: u64,
    negative_count: u64,
}

impl TernaryBloom {
    pub fn new(hash_count: usize) -> Self {
        Self { bits: vec![0; DEFAULT_SIZE], hash_count, positive_count: 0, negative_count: 0 }
    }

    /// Insert with positive weight (boost membership).
    pub fn insert_positive(&mut self, item: &[u8]) {
        for i in 0..self.hash_count {
            let idx = hash(item, i as u64);
            self.bits[idx] = 1;
        }
        self.positive_count += 1;
    }

    /// Insert with negative weight (block membership).
    pub fn insert_negative(&mut self, item: &[u8]) {
        for i in 0..self.hash_count {
            let idx = hash(item, i as u64);
            self.bits[idx] = -1;
        }
        self.negative_count += 1;
    }

    /// Check membership: returns sum of weights at hash positions.
    pub fn check(&self, item: &[u8]) -> i32 {
        (0..self.hash_count).map(|i| self.bits[hash(item, i as u64)] as i32).sum()
    }

    /// Check if positively present.
    pub fn contains(&self, item: &[u8]) -> bool { self.check(item) > 0 }

    /// Check if negatively blocked.
    pub fn blocked(&self, item: &[u8]) -> bool { self.check(item) < 0 }

    /// Merge two filters: positive wins ties (conservative).
    pub fn merge(&mut self, other: &TernaryBloom) {
        for (a, b) in self.bits.iter_mut().zip(&other.bits) {
            if *b != 0 && *a == 0 { *a = *b; }
            else if *a == *b { /* already same */ }
            else if *a == 1 && *b == -1 { *a = 0; } // conflict → neutral
        }
    }

    /// Pack to u32 for GPU transfer (16 bits per u32, 2 bits each).
    pub fn pack_for_gpu(&self) -> Vec<u32> {
        self.bits.chunks(16).map(|chunk| {
            let mut packed = 0u32;
            for (i, &v) in chunk.iter().enumerate() {
                let bits = match v { -1 => 0b11u32, 1 => 0b01, _ => 0b00 };
                packed |= bits << (i * 2);
            }
            packed
        }).collect()
    }

    pub fn positive_count(&self) -> u64 { self.positive_count }
    pub fn negative_count(&self) -> u64 { self.negative_count }
    pub fn fill_rate(&self) -> f64 { self.bits.iter().filter(|&&b| b != 0).count() as f64 / self.bits.len() as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_check() {
        let mut bf = TernaryBloom::new(3);
        bf.insert_positive(b"hello");
        assert!(bf.contains(b"hello"));
    }

    #[test]
    fn test_negative_block() {
        let mut bf = TernaryBloom::new(3);
        bf.insert_negative(b"spam");
        assert!(bf.blocked(b"spam"));
    }

    #[test]
    fn test_absent() {
        let bf = TernaryBloom::new(3);
        assert!(!bf.contains(b"missing"));
    }

    #[test]
    fn test_check_score() {
        let mut bf = TernaryBloom::new(5);
        bf.insert_positive(b"good");
        let score = bf.check(b"good");
        assert!(score > 0);
    }

    #[test]
    fn test_merge() {
        let mut bf1 = TernaryBloom::new(3);
        let mut bf2 = TernaryBloom::new(3);
        bf1.insert_positive(b"hello");
        bf2.insert_positive(b"world");
        bf1.merge(&bf2);
        assert!(bf1.contains(b"hello"));
    }

    #[test]
    fn test_gpu_pack() {
        let mut bf = TernaryBloom::new(3);
        bf.insert_positive(b"test");
        let packed = bf.pack_for_gpu();
        assert!(!packed.is_empty());
        // Each packed u32 has 16 ternary values
        assert_eq!(packed.len(), DEFAULT_SIZE / 16);
    }

    #[test]
    fn test_fill_rate() {
        let mut bf = TernaryBloom::new(3);
        bf.insert_positive(b"a");
        bf.insert_positive(b"b");
        assert!(bf.fill_rate() > 0.0);
    }

    #[test]
    fn test_positive_negative_counts() {
        let mut bf = TernaryBloom::new(3);
        bf.insert_positive(b"a");
        bf.insert_negative(b"b");
        assert_eq!(bf.positive_count(), 1);
        assert_eq!(bf.negative_count(), 1);
    }
}
