use primitives::{script::*, varint::VarInt, transaction::Transaction};
use bytes::{Bytes, Buf, BufMut, IntoBuf};
pub trait TryFrom<T>: Sized {
    type Err;
    fn try_from(_: T) -> Result<Self, Self::Err>;
} // Isn't this stable now??

impl From<VarInt> for Bytes {
    fn from(varint: VarInt) -> Bytes {
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
}

// TODO: Catch errors properly
impl TryFrom<Bytes> for VarInt {
    type Err = String;
    // TODO: Catch errors properly
    fn try_from(raw: Bytes) -> Result<VarInt, Self::Err> {
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

impl From<Transaction> for Bytes {
    fn from(tx: Transaction) -> Bytes {
        let mut pass_by_u8: u8 = 0;
        let mut buf = vec![];
        let mut exp = 0;
        let scripts = Vec::from(tx);

        for script in &scripts {
            match script.get_pass_by() {
                PassBy::Value => pass_by_u8 += 1 << exp, // Can be done faster?
                PassBy::Reference => ()
            }
            exp +=1;
        }
        buf.put(pass_by_u8);

        for script in scripts {
            let script_raw = Bytes::from(script);
            buf.put(&Bytes::from(&[script_raw.len() as u8][..]));
            buf.put(&script_raw);
        }
        Bytes::from(buf)
    }
}

impl TryFrom<Bytes> for Transaction {
    type Err = String;
    fn try_from(raw: Bytes) -> Result<Transaction, Self::Err> {
        // TODO: Catch exceptions here and look for optimization
        let mut scripts = Vec::new();
        let mut buf = raw.into_buf();

        let pass_profile = VarInt::parse(buf.bytes());
        println!("Pass profile length: {}", pass_profile.len());
        buf.advance(pass_profile.len());
        let pass_profile = u64::from(pass_profile); //This limits number of scripts to 64
        println!("Pass profile length: {}", pass_profile);
        let mut exp = 0;
        loop {
            let len = buf.get_u8();

            let mut dst = vec![0; len as usize];
            buf.copy_to_slice(&mut dst);

            let script = Script::new(   
                if (pass_profile >> exp) % 2 == 1 { 
                    PassBy::Value 
                } else { 
                    PassBy::Reference
                }, 
                Bytes::from(dst) 
            );
            scripts.push(script);

            if !buf.has_remaining() { break }
            exp += 1;
        }
        Ok(Transaction::new(scripts))
    }
}
