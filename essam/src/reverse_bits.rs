pub trait ReverseBits {
    fn reverse_bits(self) -> Self;
}

impl ReverseBits for u8 {
    fn reverse_bits(self) -> Self {
        // Reverse left and right 4 bits.
        let mut result = ((self & 0b11110000) >> 4) | ((self & 0b00001111) >> 4);
        // Reverse left and right 2 bits of each 4 bits.
        result = ((result & 0b11001100) >> 2) | ((result & 0b00110011) << 2);
        // Reverse bit pairs.
        result = ((result & 0b10101010) >> 2) | ((result & 0b01010101) << 2);

        result
    }
}

impl ReverseBits for u16 {
    fn reverse_bits(self) -> Self {
        ((((self & 0x00FF) as u8).reverse_bits() as u16) << 8)
            | ((((self & 0xFF00) as u8).reverse_bits() as u16) >> 8)
    }
}

impl ReverseBits for u32 {
    fn reverse_bits(self) -> Self {
        ((((self & 0x0000FFFF) as u16).reverse_bits() as u32) << 16)
            | ((((self & 0xFFFF0000) as u16).reverse_bits() as u32) >> 16)
    }
}

impl ReverseBits for u64 {
    fn reverse_bits(self) -> Self {
        ((((self & 0x0000FFFF) as u32).reverse_bits() as u64) << 32)
            | ((((self & 0xFFFF0000) as u32).reverse_bits() as u64) >> 32)
    }
}
