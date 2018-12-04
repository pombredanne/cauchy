// THis needs fixing
const WINDOW_LEN: usize = 64;
const HASH_LEN: usize = 32;
const SIG_LEN: usize = 32;

struct HashFunction(
    Fn([u8; WINDOW_LEN]) -> [u8; HASH_LEN]
);