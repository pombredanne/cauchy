use bytes::Bytes;

use crypto::hashes::*;

pub fn get_bit_pos<T>(value: &T, modulo: usize) -> (u16, usize)
where
    T: Identifiable,
{
    let digest = value.get_id();
    get_id_bit_pos(&digest, modulo)
}

pub fn get_id_bit_pos(value: &Bytes, modulo: usize) -> (u16, usize) {
    let modulo = modulo as u16;
    let pos = (u16::from(value[0]) + (u16::from(value[1]) >> 8)) % (modulo * 8); // Bit position // TODO: Check
    let shift = &pos % 8; // Position within the byte
    let index = (pos / 8) as usize; // Position of the byte
    (shift, index)
}
