use primitives::{script::Script, varint::VarInt, transaction::Transaction};
use bytes::{Bytes, BytesMut, Buf, BufMut, IntoBuf};

pub trait SerialisableType<T> {
    fn serialise(T) -> Bytes;
    fn deserialise(raw: Bytes) -> Result<T, String>;
}

impl SerialisableType<Script> for Script {
    fn serialise(script: Script) -> Bytes {
        Bytes::from(script)
    }

    fn deserialise(raw: Bytes) -> Result<Script, String> {
        // TODO: Is this right? Should we assume that the varint is in the raw too?
        Ok(Script::from(raw.clone()))
    }
}

impl SerialisableType<VarInt> for VarInt {
    fn serialise(varint: VarInt) -> Bytes {
        let mut n = u64::from(varint);
        let mut tmp = vec![];
        let mut len = 0;
        loop {
            tmp.put((0x7f & n) as u8 | (if len == 0 {0x00} else {0x80}));
            if n <= 0x7f { break; } 
            n = (n >> 7) - 1;
            len += 1;
        }
        tmp.reverse();
        Bytes::from(tmp)
        }

    fn deserialise(raw: Bytes) -> Result<VarInt, String> {
        let mut n: u64 = 0;
        let mut buf = raw.into_buf();
        loop {
            let k = buf.get_u8();
            n = (n << 7) | ((k & 0x7f) as u64);
            if 0x00 != (k & 0x80) {
                n += 1;
            } else {
                return Ok(VarInt::new(n));
            }
        }
    }
}

impl SerialisableType<Transaction> for Transaction {
    fn serialise(tx: Transaction) -> Bytes {
        let n_instructions = usize::from(tx.n_instructions.clone());
        let mut serial = BytesMut::with_capacity(n_instructions);

        serial.put(VarInt::serialise(tx.n_instructions));
        serial.put(Bytes::from(tx.instructions));
        serial.put(tx.memory);

        serial.freeze()
    }

    fn deserialise(raw: Bytes) -> Result<Transaction, String> {
        let n_instructions = VarInt::parse(&raw);
        let n_instruc_len = n_instructions.len();
        let n_instructions = usize::from(n_instructions);

        if raw.len() < n_instruc_len + n_instructions {
            return Err("Data too short to deserialise".to_string())
        }

        let instructions = match Script::deserialise(raw.slice(n_instruc_len, n_instructions + n_instruc_len)) {
            Ok(some) => some,
            Err(some) => return Err(some)
        };

        Ok(Transaction { 
            n_instructions: VarInt::from(n_instructions), // Trade off here
            instructions: instructions, 
            memory: raw.slice_from(n_instructions + n_instruc_len), 
            })
    }
}