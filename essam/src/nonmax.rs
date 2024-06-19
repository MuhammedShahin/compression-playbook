use std::num::{NonZeroU16, NonZeroU32, NonZeroU64, NonZeroUsize};

// TODO: Learn about macros to generalize this struct
#[derive(Clone, Copy)]
pub struct NonMaxU16(NonZeroU16);

#[derive(Clone, Copy)]
pub struct NonMaxU32(NonZeroU32);

#[derive(Clone, Copy)]
pub struct NonMaxU64(NonZeroU64);

#[derive(Clone, Copy)]
pub struct NonMaxUsize(NonZeroUsize);

impl NonMaxU16 {
    pub fn new(value: u16) -> Option<Self> {
        NonZeroU16::new(!value).map(Self)
    }

    pub fn get(&self) -> u16 {
        !self.0.get()
    }
}

impl NonMaxU32 {
    pub fn new(value: u32) -> Option<Self> {
        NonZeroU32::new(!value).map(Self)
    }

    pub fn get(&self) -> u32 {
        !self.0.get()
    }
}

impl NonMaxU64 {
    pub fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(!value).map(Self)
    }

    pub fn get(&self) -> u64 {
        !self.0.get()
    }
}

impl NonMaxUsize {
    pub fn new(value: usize) -> Option<Self> {
        NonZeroUsize::new(!value).map(Self)
    }

    pub fn get(&self) -> usize {
        !self.0.get()
    }
}
