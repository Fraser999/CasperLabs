//! Provides encoding to and decoding from the CasperLabs binary format.
mod decode;
mod encode;
mod error;

use alloc::vec::Vec;

use decode::Deserializer;
use encode::{Serializer, SizeChecker};
pub use error::{Error, Result};

/// Serializes a serializable object into a `Vec` of bytes.
pub fn serialize<T: serde::Serialize + ?Sized>(value: &T) -> Result<Vec<u8>> {
    let serialized_length = serialized_length(value)?;
    let mut serializer = Serializer::new(serialized_length);
    value.serialize(&mut serializer)?;
    Ok(serializer.take_output())
}

/// Deserializes a slice of bytes into an instance of `T`.
pub fn deserialize<'a, T: serde::Deserialize<'a>>(bytes: &'a [u8]) -> Result<T> {
    let mut deserializer = Deserializer::new(bytes)?;
    let result = T::deserialize(&mut deserializer)?;
    deserializer.input_slice_is_empty()?;
    Ok(result)
}

/// Returns the size that an object would be if serialized.
pub fn serialized_length<T: serde::Serialize + ?Sized>(value: &T) -> Result<u32> {
    let mut size_checker = SizeChecker::new();

    value.serialize(&mut size_checker)?;
    Ok(size_checker.take_total())
}
