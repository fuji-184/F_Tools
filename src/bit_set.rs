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