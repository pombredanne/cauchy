// ECDSA Errors
#[derive(Debug, Fail)]
#[fail(display = "invalid signature")]
pub struct InvalidSignature;

#[derive(Debug, Fail)]
#[fail(display = "invalid pubkey")]
pub struct InvalidPubkey;

// Parsing Errors
#[derive(Debug, Fail)]
#[fail(display = "varint parsing ran out of bytes at {}", len)]
pub struct VarIntParseError {
    pub len: usize,
}

// Decode errors
#[derive(Debug, Fail)]
pub enum CryptoDecodingError {
    #[fail(display = "invalid pubkey")]
    InvalidPubkey,
    #[fail(display = "invalid signature")]
    InvalidSignature,
}

#[derive(Debug, Fail)]
#[fail(display = "malformed message")]
pub struct MalformedMessageError;

#[derive(Debug)]
pub enum DecodingError {
    StartHandshakeError,
    EndHandshakeError,
    NonceError,
    MiniSketchError,
    GetTransactionsError,
    TransactionsError,
    InvalidMessage,
    IOError,
}

// Serialisation Errors
#[derive(Debug, Fail)]
pub enum TransactionDeserialisationError {
    #[fail(display = "invalid time varint")]
    TimeVarInt,
    #[fail(display = "invalid aux varint")]
    AuxVarInt,
    #[fail(display = "invalid binary varint")]
    BinaryVarInt,
    #[fail(display = "auxilary too short")]
    AuxTooShort,
    #[fail(display = "binary too short")]
    BinaryTooShort,
}

#[derive(Debug, Fail)]
#[fail(display = "invalid varint")]
pub struct VarIntDeserialisationError;

// Heartbeat Errors
#[derive(Debug, Fail)]
#[fail(display = "odd sketch heart failure")]
pub struct HeartBeatOddSketchError;

#[derive(Debug, Fail)]
#[fail(display = "nonce heart failure")]
pub struct HeartBeatNonceError;

#[derive(Debug, Fail)]
#[fail(display = "impulse send failure")]
pub struct ImpulseSendError;

#[derive(Debug, Fail)]
#[fail(display = "impulse receive failure")]
pub struct ImpulseReceiveError;

// Database Errors
#[derive(Debug, Fail)]
pub enum TransactionStorageError {
    #[fail(display = "tx deserialisation error")]
    DeserialisationError,
    #[fail(display = "database error")]
    DatabaseError,
}

#[derive(Debug, Fail)]
pub enum DatabaseError {
    #[fail(display = "invalid path")]
    DbPath,
    #[fail(display = "failed to open db")]
    Open,
    #[fail(display = "failed to get item db")]
    Get,
    #[fail(display = "failed to put item db")]
    Put,
}

// Connection Errors
#[derive(Debug)]
pub struct ConnectionAddError;
