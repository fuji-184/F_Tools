
/*
High-performance boolean flag container using packed bit-fields.

This structure maps 64 independent boolean states onto a single 64-bit integer, providing 
extremely dense storage and hardware-accelerated set, clear, and query operations. It is 
primarily used to optimize state management—such as tracking task completion statuses 
in parallel pipelines, managing resource allocation masks for worker pools, or implementing 
fast Bloom filter primitives—by drastically reducing memory bandwidth requirements and 
leveraging CPU-level bitwise instructions for near-instant status checking.
*/

pub struct BitSet(u64);

impl BitSet {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn set(&mut self, bit: usize) {
        self.0 |= 1 << bit;
    }

    pub fn clear(&mut self, bit: usize) {
        self.0 &= !(1 << bit);
    }

    pub fn is_set(&self, bit: usize) -> bool {
        (self.0 >> bit) & 1 == 1
    }

    pub fn count_set(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn find_unset(&self) -> Option<usize> {
        let inverted = !self.0;
        if inverted == 0 {
            None
        } else {
            Some(inverted.trailing_zeros() as usize)
        }
    }
}

ftest::test!(bit_set_tests, {
    test_basic_operations {
        let mut bitset = BitSet::new();

        assert!(!bitset.is_set(3));
        assert_eq!(bitset.count_set(), 0);

        bitset.set(3);
        bitset.set(5);

        assert!(bitset.is_set(3));
        assert!(bitset.is_set(5));
        assert!(!bitset.is_set(4));
        assert_eq!(bitset.count_set(), 2);

        bitset.clear(3);
        assert!(!bitset.is_set(3));
        assert!(bitset.is_set(5));
        assert_eq!(bitset.count_set(), 1);
    }

    test_find_unset {
        let mut bitset = BitSet::new();
        assert_eq!(bitset.find_unset(), Some(0));

        bitset.set(0);
        bitset.set(1);
        assert_eq!(bitset.find_unset(), Some(2));

        bitset.0 = u64::MAX;
        assert_eq!(bitset.find_unset(), None);
    }
});