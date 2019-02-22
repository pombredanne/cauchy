use crypto::hashes::blake2b::Blk2bHashable;
use bytes::Bytes;

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

pub fn get_id_bit_pos(value: &Bytes, modulo: usize) -> (u16, usize)
{
    let modulo = modulo as u16;
    let pos = (u16::from(value[0]) + (u16::from(value[1]) >> 8)) % (modulo * 8); // Bit position // TODO: Check
    let shift = &pos % 8; // Position within the byte
    let index = (pos / 8) as usize; // Position of the byte
    (shift, index)
}
