use thiserror::Error;

#[derive(Error, Debug)]
pub enum DictModificationError {
    #[error("Failed to write data due to: {0}")]
    WriteFailed(#[from] std::io::Error),
    #[error("The word '{0}' already exists in the given dictionary")]
    WordAlreadyExists(String),

    #[error("The '{0}' dict ID is not present in the configuration")]
    UnknownDictID(String),
}
