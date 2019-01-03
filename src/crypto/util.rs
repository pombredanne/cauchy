use crypto::hashes::blake2b::Blk2bHashable;

pub fn get_pos<T>(value: &T, seed: usize, modulo: usize) -> usize
where
    T: Blk2bHashable,
{
    let pos = value.blake2b()[seed] as usize;
    pos % modulo
}

pub fn get_bit_pos<T>(value: &T, modulo: usize) -> (u8, usize)
where
    T: Blk2bHashable,
{
    let pos = value.blake2b()[0];
    let shift = &pos % 8; // Position within the byte
    let index = (pos / (modulo >> 3) as u8) as usize; // Position of the byte
    (shift, index)
}
