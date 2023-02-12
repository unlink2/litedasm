use thiserror::Error;

use super::dasm::Address;

pub type FdResult<T> = Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unknown error")]
    Unknown,
    #[error("Transform out of data")]
    TransformOutOfData(Address),
    #[error("Unable to match any pattern")]
    NoMatch,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
