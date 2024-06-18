type ElementType = u64;
const NUM_BITS: usize = std::mem::size_of::<ElementType>() * 8;

pub struct Bitset {
    data: Vec<ElementType>,
}

struct BitsetIterator<'a> {
    bitset: &'a Bitset,
    current_data: ElementType,
    data_idx: usize,
}

impl std::fmt::Debug for Bitset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.iter().collect::<Vec<_>>())
    }
}

impl Bitset {
    pub fn with_capacity(capacity: usize) -> Self {
        let len = (capacity + NUM_BITS - 1) / NUM_BITS;
        Bitset { data: vec![0; len] }
    }

    pub fn contains(&self, value: &usize) -> bool {
        let byte_idx = value / NUM_BITS;
        let bit_idx = value % NUM_BITS;

        if byte_idx >= self.data.len() {
            false
        } else {
            (self.data[byte_idx] & ((1 as ElementType) << bit_idx)) != 0
        }
    }

    pub fn insert(&mut self, value: usize) {
        let byte_idx = value / NUM_BITS;
        let bit_idx = value % NUM_BITS;

        if byte_idx >= self.data.len() {
            self.data.resize(byte_idx + 1, 0);
        }

        self.data[byte_idx] |= (1 as ElementType) << bit_idx;
    }

    pub fn set(&mut self, value: usize) {
        self.insert(value)
    }

    pub fn remove(&mut self, value: usize) {
        let byte_idx = value / NUM_BITS;
        let bit_idx = value % NUM_BITS;

        if byte_idx >= self.data.len() {
            return;
        }

        self.data[byte_idx] &= !((1 as ElementType) << bit_idx);
    }

    pub fn clear(&mut self, value: usize) {
        self.remove(value)
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

    pub fn count_ones(&self) -> usize {
        let mut count = 0;
        for element in &self.data {
            count += element.count_ones() as usize;
        }
        count
    }

    pub fn count_ones_sliced(&self, from: usize, to: usize) -> usize {
        debug_assert!(to >= from);

        let byte_from = from / NUM_BITS;
        let byte_to = to / NUM_BITS;
        let bit_from = (from % NUM_BITS) as u32;
        let bit_to = (to % NUM_BITS) as u32; // exclusive

        let all_ones = !(0 as ElementType);
        let first_byte_mask = all_ones.overflowing_shl(bit_from).0;
        // Shift value here can be 64, which is behaves differently, so split the shift to two parts.
        let last_byte_mask = all_ones
            .overflowing_shr(NUM_BITS as u32 - bit_to - 1)
            .0
            .overflowing_shr(1)
            .0;

        let mut count = 0;
        if byte_to > byte_from {
            // Count bits from the first byte
            count += (self.data[byte_from] & first_byte_mask).count_ones() as usize;
            // Count bits from the last byte
            count += (self.data[byte_to] & last_byte_mask).count_ones() as usize;

            for element in &self.data[(byte_from + 1)..byte_to] {
                count += element.count_ones() as usize;
            }
        } else {
            count +=
                (self.data[byte_from] & first_byte_mask & last_byte_mask).count_ones() as usize;
        }

        count
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

        let bit_idx = NUM_BITS - self.current_data.leading_zeros() as usize - 1;
        self.current_data &= !((1 as ElementType) << bit_idx);

        Some(self.data_idx * NUM_BITS + bit_idx)
    }
}
