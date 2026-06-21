use alloc::string::String;

#[derive(Debug)]
pub enum DecodeError {
    KaniStubError,
    InvalidField(String),
    InvalidLength(String),
    InvalidPacketType(String),
    UnexpectedEof(String),
    InvalidData(String),
    InvalidDataTokenType(String),
    InvalidEnvChangeType(String),
    InvalidColMetaData(String)
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DecodeError::KaniStubError => write!(f, ""),
            DecodeError::InvalidField(x) => write!(f, "Invalid field: {}", x),
            DecodeError::InvalidLength(x) => write!(f, "Invalid length: {}", x),
            DecodeError::InvalidPacketType(x) => write!(f, "Invalid packet type: {}", x),
            DecodeError::UnexpectedEof(x) => write!(f, "Unexpected end of data: {}", x),
            DecodeError::InvalidData(x) => write!(f, "Invalid data: {}", x),
            DecodeError::InvalidDataTokenType(x) => write!(f, "Invalid data token type: {}", x),
            DecodeError::InvalidEnvChangeType(x) => write!(f, "Invalid env change type: {}", x),
            DecodeError::InvalidColMetaData(x) => write!(f, "Invalid column metadata: {}", x),
        }
    }
}

impl core::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        None
    }
}
