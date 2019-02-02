use crypto::hashes::blake2b::Blk2bHashable;

pub fn get_pos<T>(value: &T, seed: usize, modulo: usize) -> usize
where
    T: Blk2bHashable,
{
    let pos = value.blake2b()[seed] as usize;
    pos % modulo
}

pub fn get_bit_pos<T>(value: &T, modulo: usize) -> (u16, usize)
where
    T: Blk2bHashable,
{
    let digest = value.blake2b();
    let modulo = modulo as u16;
    let pos = ((digest[0] as u16) + ((digest[1] as u16) << 8)) % (modulo * 8); // Bit position // TODO: Check
    let shift = &pos % 8; // Position within the byte
    let index = (pos / modulo) as usize; // Position of the byte
    (shift, index)
}
