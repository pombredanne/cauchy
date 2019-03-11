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
#[fail(display = "work heart failure")]
pub struct HeartBeatWorkError;

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
pub enum SystemError {
    #[fail(display = "invalid path")]
    InvalidPath,
}

#[derive(Debug, Fail)]
pub enum ArenaError {
    #[fail(display = "failed to push perception to local")]
    PushLocal,
}

// Connection Errors
#[derive(Debug, Fail)]
#[fail(display = "socket not found")]
pub struct SocketNotFound;

// Connection Errors
#[derive(Debug, Fail)]
#[fail(display = "socket not found")]
pub struct ConnectionAddError;

#[derive(Debug, Fail)]
pub enum DaemonError {
    #[fail(display = "socket binding failure")]
    BindFailure,
    #[fail(display = "socket binding failure: {}", err)]
    SocketAcceptanceFailure { err: std::io::Error },
    #[fail(display = "new peer stream error: {}", err)]
    NewSocketError { err: std::io::Error },
    #[fail(display = "no perception found")]
    Perceptionless,
    #[fail(display = "missing transaction")]
    MissingTransaction,
    #[fail(display = "unreachable")]
    Unreachable,
}
