use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("invalid resource kind: {0}")]
    InvalidResourceKind(u8),
    #[error("invalid texture format: {0}")]
    InvalidTextureFormat(u8),
}

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("data hash doesn't match expected hash")]
    InvalidHash,

    #[error(transparent)]
    Format(#[from] FormatError),
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    BincodeEncode(#[from] bincode::error::EncodeError),
    #[error(transparent)]
    BincodeDecode(#[from] bincode::error::DecodeError),
}
