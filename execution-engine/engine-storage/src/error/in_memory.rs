use std::sync;

use failure::Fail;

use types::encoding;

#[derive(Debug, Fail, PartialEq, Eq)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Encoding(#[fail(cause)] encoding::Error),

    #[fail(display = "Another thread panicked while holding a lock")]
    Poison,
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
