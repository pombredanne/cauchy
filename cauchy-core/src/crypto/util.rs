use crypto::hashes::blake2b::Blk2bHashable;

pub fn get_bit_pos<T>(value: &T, modulo: usize) -> (u16, usize)
where
    T: Blk2bHashable,
{
    let digest = value.blake2b().blake2b();
    let modulo = modulo as u16;
    let pos = (u16::from(digest[0]) + (u16::from(digest[1]) >> 8)) % (modulo * 8); // Bit position // TODO: Check
    let shift = &pos % 8; // Position within the byte
    let index = (pos / 8) as usize; // Position of the byte
    (shift, index)
}
