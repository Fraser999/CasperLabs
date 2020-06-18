use alloc::vec::Vec;
use core::fmt;

use failure::Fail;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{encoding, CLType, CLTyped};

/// Error while converting a [`CLValue`] into a given type.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CLTypeMismatch {
    /// The [`CLType`] into which the `CLValue` was being converted.
    pub expected: CLType,
    /// The actual underlying [`CLType`] of this `CLValue`, i.e. the type from which it was
    /// constructed.
    pub found: CLType,
}

impl fmt::Display for CLTypeMismatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "Expected {:?} but found {:?}.",
            self.expected, self.found
        )
    }
}

/// Error relating to [`CLValue`] operations.
#[derive(Fail, PartialEq, Eq, Clone, Debug)]
pub enum CLValueError {
    /// An error while serializing or deserializing the underlying data.
    #[fail(display = "Encoding error: {}", _0)]
    Serialization(encoding::Error),
    /// A type mismatch while trying to convert a [`CLValue`] into a given type.
    #[fail(display = "Type mismatch: {}", _0)]
    Type(CLTypeMismatch),
}

/// A CasperLabs value, i.e. a value which can be stored and manipulated by smart contracts.
///
/// It holds the underlying data as a type-erased, serialized `Vec<u8>` and also holds the
/// [`CLType`] of the underlying data as a separate member.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct CLValue {
    #[serde(with = "serde_bytes")]
    bytes: Vec<u8>,
    cl_type: CLType,
}

impl CLValue {
    /// Constructs a `CLValue` from `t`.
    pub fn from_t<T: CLTyped + Serialize>(t: T) -> Result<CLValue, CLValueError> {
        let bytes = encoding::serialize(&t).map_err(CLValueError::Serialization)?;

        Ok(CLValue {
            cl_type: T::cl_type(),
            bytes,
        })
    }

    /// Consumes and converts `self` back into its underlying type.
    pub fn into_t<T: CLTyped + DeserializeOwned>(self) -> Result<T, CLValueError> {
        let expected = T::cl_type();

        if self.cl_type == expected {
            encoding::deserialize(&self.bytes).map_err(CLValueError::Serialization)
        } else {
            Err(CLValueError::Type(CLTypeMismatch {
                expected,
                found: self.cl_type,
            }))
        }
    }

    // This is only required in order to implement `TryFrom<state::CLValue> for CLValue` (i.e. the
    // conversion from the Protobuf `CLValue`) in a separate module to this one.
    #[doc(hidden)]
    pub fn from_components(cl_type: CLType, bytes: Vec<u8>) -> Self {
        Self { cl_type, bytes }
    }

    // This is only required in order to implement `From<CLValue> for state::CLValue` (i.e. the
    // conversion to the Protobuf `CLValue`) in a separate module to this one.
    #[doc(hidden)]
    pub fn destructure(self) -> (CLType, Vec<u8>) {
        (self.cl_type, self.bytes)
    }

    /// The [`CLType`] of the underlying data.
    pub fn cl_type(&self) -> &CLType {
        &self.cl_type
    }

    /// Returns a reference to the serialized form of the underlying value held in this `CLValue`.
    pub fn inner_bytes(&self) -> &Vec<u8> {
        &self.bytes
    }
}
