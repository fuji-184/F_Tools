use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub struct BloomFilter {
    bits: Vec<u64>,
    num_hashes: u8,
    cap_bits: usize,
}

impl BloomFilter {
    pub fn new(size_bits: usize, num_hashes: u8) -> Self {
        let num_elements = (size_bits + 63) / 64;
        Self {
            bits: vec![0u64; num_elements],
            num_hashes,
            cap_bits: size_bits,
        }
    }

    fn get_base_hashes<T: Hash>(&self, item: &T) -> (u64, u64) {
        let mut h1 = DefaultHasher::new();
        item.hash(&mut h1);
        let h1_res = h1.finish();

        let mut h2 = DefaultHasher::new();
        h1_res.hash(&mut h2);
        let h2_res = h2.finish();

        (h1_res, h2_res)
    }

    pub fn insert<T: Hash>(&mut self, item: &T) {
        let (h1, h2) = self.get_base_hashes(item);
        for i in 0..self.num_hashes {
            let combined_hash = h1.wrapping_add((i as u64).wrapping_mul(h2));
            let idx = (combined_hash as usize) % self.cap_bits;
            
            let bucket = idx / 64;
            let bit = idx % 64;
            self.bits[bucket] |= 1 << bit;
        }
    }

    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        let (h1, h2) = self.get_base_hashes(item);
        for i in 0..self.num_hashes {
            let combined_hash = h1.wrapping_add((i as u64).wrapping_mul(h2));
            let idx = (combined_hash as usize) % self.cap_bits;
            
            let bucket = idx / 64;
            let bit = idx % 64;
            if (self.bits[bucket] & (1 << bit)) == 0 {
                return false;
            }
        }
        true
    }
}