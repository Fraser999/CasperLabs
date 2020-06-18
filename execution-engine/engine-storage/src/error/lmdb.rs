use std::sync;

use failure::Fail;
use lmdb as lmdb_external;

use types::encoding;

use super::in_memory;

#[derive(Debug, Clone, Fail, PartialEq, Eq)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Lmdb(#[fail(cause)] lmdb_external::Error),

    #[fail(display = "{}", _0)]
    Encoding(#[fail(cause)] encoding::Error),

    #[fail(display = "Another thread panicked while holding a lock")]
    Poison,
}

impl wasmi::HostError for Error {}

impl From<lmdb_external::Error> for Error {
    fn from(error: lmdb_external::Error) -> Self {
        Error::Lmdb(error)
    }
}

impl From<encoding::Error> for Error {
    fn from(error: encoding::Error) -> Self {
        Error::Encoding(error)
    }
}

impl<T> From<sync::PoisonError<T>> for Error {
    fn from(_error: sync::PoisonError<T>) -> Self {
        Error::Poison
    }
}

impl From<in_memory::Error> for Error {
    fn from(error: in_memory::Error) -> Self {
        match error {
            in_memory::Error::Encoding(error) => Error::Encoding(error),
            in_memory::Error::Poison => Error::Poison,
        }
    }
}
