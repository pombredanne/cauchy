use std::ops::Add;

#[derive(Clone)]
pub struct VarInt(u64);

impl VarInt {
    pub fn new(n: u64) -> VarInt {
        VarInt(n)
    }

    pub fn len(&self) -> usize {
        let mut n = self.0;
        let mut n_ret: usize = 0;
        loop {
            n_ret += 1;
            if n <= 0x7F {
                break;
            }
            n = (n >> 7) - 1;
        }
        n_ret
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

impl Add for VarInt {
    type Output = VarInt;

    fn add(self, other: VarInt) -> VarInt {
        VarInt(self.0 + u64::from(other))
    }
}

impl From<usize> for VarInt {
    fn from(item: usize) -> Self {
        VarInt(item as u64)
    }
}

impl From<VarInt> for usize {
    fn from(item: VarInt) -> Self {
        item.0 as usize
    }
}

impl From<u64> for VarInt {
    fn from(item: u64) -> Self {
        VarInt(item)
    }
}

impl From<u32> for VarInt {
    fn from(item: u32) -> Self {
        VarInt(u64::from(item))
    }
}

impl From<VarInt> for u32 {
    fn from(item: VarInt) -> Self {
        item.0 as u32
    }
}

impl From<VarInt> for u64 {
    fn from(item: VarInt) -> Self {
        item.0 as u64
    }
}
