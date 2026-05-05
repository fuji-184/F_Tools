use std::sync::atomic::{AtomicU64, Ordering};

pub struct AtomicBitSet {
    bits: Vec<AtomicU64>,
}

impl AtomicBitSet {
    pub fn new(size: usize) -> Self {
        let num_elements = (size + 63) / 64;
        let mut bits = Vec::with_capacity(num_elements);
        for _ in 0..num_elements {
            bits.push(AtomicU64::new(0));
        }
        Self { bits }
    }

    pub fn set(&self, index: usize) {
        let bucket = index / 64;
        let bit = index % 64;
        if bucket < self.bits.len() {
            self.bits[bucket].fetch_or(1 << bit, Ordering::Relaxed);
        }
    }

    pub fn unset(&self, index: usize) {
        let bucket = index / 64;
        let bit = index % 64;
        if bucket < self.bits.len() {
            self.bits[bucket].fetch_and(!(1 << bit), Ordering::Relaxed);
        }
    }

    pub fn check(&self, index: usize) -> bool {
        let bucket = index / 64;
        let bit = index % 64;
        if bucket < self.bits.len() {
            (self.bits[bucket].load(Ordering::Relaxed) & (1 << bit)) != 0
        } else {
            false
        }
    }
}