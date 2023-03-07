use thiserror::Error;

use super::dasm::Address;
use super::dasm::ValueTypeFmt;

pub type FdResult<T> = Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unknown error")]
    Unknown,
    #[error("Transform out of data")]
    TransformOutOfData(Address),
    #[error("Unable to match any pattern")]
    NoMatch,
    #[error("Transform was not found")]
    TransformNotFound(String),
    #[error("Unsupported format")]
    UnsupportedFormat(ValueTypeFmt),
    #[error("Arch not found")]
    ArchNotFound(String),
    #[error("Unable to patch file")]
    PatchOffsetOutOfRange(usize),
    #[error("Label not found")]
    LabelNotFound(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
