pub struct Bitset {
    data: Vec<u64>,
}

struct BitsetIterator<'a> {
    bitset: &'a Bitset,
    current_data: u64,
    data_idx: usize,
}

impl std::fmt::Debug for Bitset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.iter().collect::<Vec<_>>())
    }
}

impl Bitset {
    pub(self) const NUM_BITS: usize = std::mem::size_of::<u64>() * 8;

    pub fn with_capacity(capacity: usize) -> Self {
        let len = (capacity + Self::NUM_BITS - 1) / Self::NUM_BITS;
        Bitset { data: vec![0; len] }
    }

    pub fn contains(&self, value: &usize) -> bool {
        let byte_idx = value / Self::NUM_BITS;
        let bit_idx = value % Self::NUM_BITS;

        if byte_idx >= self.data.len() {
            false
        } else {
            (self.data[byte_idx] & ((1 as u64) << bit_idx)) != 0
        }
    }

    pub fn insert(&mut self, value: usize) {
        let byte_idx = value / Self::NUM_BITS;
        let bit_idx = value % Self::NUM_BITS;

        if byte_idx >= self.data.len() {
            self.data.resize(byte_idx + 1, 0);
        }

        self.data[byte_idx] |= (1 as u64) << bit_idx;
    }

    pub fn remove(&mut self, value: usize) {
        let byte_idx = value / Self::NUM_BITS;
        let bit_idx = value % Self::NUM_BITS;

        if byte_idx >= self.data.len() {
            return;
        }

        self.data[byte_idx] &= !((1 as u64) << bit_idx);
    }

    pub fn extend(&mut self, rhs: &Bitset) {
        if rhs.data.len() > self.data.len() {
            self.data.resize(rhs.data.len(), 0);
        }

        for idx in 0..rhs.data.len() {
            self.data[idx] |= rhs.data[idx];
        }
    }

    pub fn iter(&self) -> impl std::iter::Iterator<Item = usize> + '_ {
        BitsetIterator {
            bitset: &self,
            current_data: if self.data.is_empty() {
                0
            } else {
                self.data[0]
            },
            data_idx: 0,
        }
    }
}

impl std::ops::BitOrAssign<&Self> for Bitset {
    fn bitor_assign(&mut self, rhs: &Self) {
        self.extend(rhs)
    }
}

impl<'a> std::iter::Iterator for BitsetIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_data == 0 {
            self.data_idx += 1;

            if self.data_idx >= self.bitset.data.len() {
                return None;
            }

            self.current_data = self.bitset.data[self.data_idx];
        }

        let bit_idx = Bitset::NUM_BITS - self.current_data.leading_zeros() as usize - 1;
        self.current_data &= !((1 as u64) << bit_idx);

        Some(self.data_idx * Bitset::NUM_BITS + bit_idx)
    }
}
