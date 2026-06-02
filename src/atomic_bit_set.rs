
/*
Thread-safe bitmap for lock-free bitwise state manipulation and flag tracking.

This structure manages an array of atomic 64-bit integers to support highly concurrent
indexing, setting, clearing, and probing of individual boolean indicators. By leveraging 
fine-grained atomic operations with relaxed memory ordering constraints, it avoids internal
lock contention across thread boundaries while minimizing footprint requirements.
*/

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

ftest::test!(atomic_bit_set_tests, {
    test_set_and_check {
        let bitset = AtomicBitSet::new(130);

        assert!(!bitset.check(5));
        assert!(!bitset.check(70));

        bitset.set(5);
        bitset.set(70);

        assert!(bitset.check(5));
        assert!(bitset.check(70));
        assert!(!bitset.check(6));
    }

    test_unset {
        let bitset = AtomicBitSet::new(64);

        bitset.set(42);
        assert!(bitset.check(42));

        bitset.unset(42);
        assert!(!bitset.check(42));
    }

    test_out_of_bounds {
        let bitset = AtomicBitSet::new(10);
        assert!(!bitset.check(20));
    }
});