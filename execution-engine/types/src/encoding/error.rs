use alloc::{
    str::Utf8Error,
    string::{String, ToString},
};
use core::fmt::{self, Display, Formatter};

use serde::ser::StdError;

/// The result of a serialization or deserialization operation.
pub type Result<T> = core::result::Result<T, Error>;

/// An error that can be produced during serialization or deserialization.
#[derive(Debug)]
pub enum Error {
    /// Returned if trying to serialize a type with more than 255 discriminants (e.g. a struct with
    /// too many fields, an enum with too many variants).
    ExcessiveDiscriminants,
    /// Returned if input slice to deserializer is too short.
    EndOfSlice,
    /// Returned if input slice has remaining bytes after deserialization complete.  Encapsulates
    /// the number of unused bytes.
    LeftOverBytes(usize),
    /// Returned if the deserializer attempts to deserialize a string that is not valid UTF-8.
    InvalidUtf8Encoding(Utf8Error),
    /// Returned if the deserializer attempts to deserialize a bool that was not encoded as either
    /// `1` or `0`.
    InvalidBoolEncoding(u8),
    /// Returned if the deserializer attempts to deserialize a char that is not in the correct
    /// format.
    InvalidCharEncoding,
    /// Returned if the deserializer attempts to deserialize the tag of an enum that is not in the
    /// expected ranges.
    InvalidTagEncoding(u8),
    /// Type to be serialized (e.g. `f32`) or serde method (e.g. `deserialize_any`) is unsupported.
    Unsupported,
    /// If (de)serializing a message takes more than the provided size limit, this
    /// error is returned.
    SizeLimit,
    /// Sequences of unknown length (like iterators) cannot be encoded.
    SequenceMustHaveLength,
    /// A custom error message from Serde.
    Custom(String),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Error::ExcessiveDiscriminants => write!(
                formatter,
                "type has more than 255 discriminants (struct fields or enum variants)"
            ),
            Error::EndOfSlice => write!(formatter, "input slice is too short"),
            Error::LeftOverBytes(count) => write!(
                formatter,
                "input slice has {} remaining bytes after deserializing",
                count
            ),
            Error::InvalidUtf8Encoding(error) => {
                write!(formatter, "string is not valid UTF-8: {}", error)
            }
            Error::InvalidBoolEncoding(bool_value) => write!(
                formatter,
                "invalid u8 while decoding bool: expected 0 or 1 but found {}",
                bool_value
            ),
            Error::InvalidCharEncoding => write!(formatter, "char is not valid UTF-8"),
            Error::InvalidTagEncoding(tag) => {
                write!(formatter, "tag for enum is not valid: found {}", tag)
            }
            Error::Unsupported => write!(
                formatter,
                "type to be serialized or serde method is unsupported"
            ),
            Error::SizeLimit => write!(formatter, "the size limit has been reached"),
            Error::SequenceMustHaveLength => write!(
                formatter,
                "sequences of unknown length (like iterators) cannot be encoded"
            ),
            Error::Custom(msg) => msg.fmt(formatter),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::InvalidUtf8Encoding(error)
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(error: T) -> Error {
        Error::Custom(error.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(error: T) -> Self {
        Error::Custom(error.to_string())
    }
}

impl StdError for Error {}
