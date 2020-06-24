use alloc::{format, string::String};
use core::{
    convert::TryFrom,
    fmt::{self, Debug, Display, Formatter},
};

use hex_fmt::HexFmt;
use serde::{
    de::{Error as SerdeError, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{AccessRights, ApiError, Key};

/// The number of bytes in a [`URef`] address.
pub const UREF_ADDR_LENGTH: usize = 32;

/// The number of bytes in a serialized [`URef`].
pub const UREF_SERIALIZED_LENGTH: usize = UREF_ADDR_LENGTH + 5;

/// Represents an unforgeable reference, containing an address in the network's global storage and
/// the [`AccessRights`] of the reference.
///
/// A `URef` can be used to index entities such as [`CLValue`](crate::CLValue)s, or smart contracts.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct URef([u8; UREF_ADDR_LENGTH], AccessRights);

impl URef {
    /// Constructs a [`URef`] from an address and access rights.
    pub fn new(address: [u8; UREF_ADDR_LENGTH], access_rights: AccessRights) -> Self {
        URef(address, access_rights)
    }

    /// Returns the address of this [`URef`].
    pub fn addr(&self) -> [u8; UREF_ADDR_LENGTH] {
        self.0
    }

    /// Returns the access rights of this [`URef`].
    pub fn access_rights(&self) -> AccessRights {
        self.1
    }

    /// Returns a new [`URef`] with the same address and updated access rights.
    pub fn with_access_rights(self, access_rights: AccessRights) -> Self {
        URef(self.0, access_rights)
    }

    /// Removes the access rights from this [`URef`].
    pub fn remove_access_rights(self) -> Self {
        URef(self.0, AccessRights::NONE)
    }

    /// Returns `true` if the access rights are `Some` and
    /// [`is_readable`](AccessRights::is_readable) is `true` for them.
    pub fn is_readable(self) -> bool {
        self.1.is_readable()
    }

    /// Returns a new [`URef`] with the same address and [`AccessRights::READ`] permission.
    pub fn into_read(self) -> URef {
        URef(self.0, AccessRights::READ)
    }

    /// Returns a new [`URef`] with the same address and [`AccessRights::READ_ADD_WRITE`]
    /// permission.
    pub fn into_read_add_write(self) -> URef {
        URef(self.0, AccessRights::READ_ADD_WRITE)
    }

    /// Returns `true` if the access rights are `Some` and
    /// [`is_writeable`](AccessRights::is_writeable) is `true` for them.
    pub fn is_writeable(self) -> bool {
        self.1.is_writeable()
    }

    /// Returns `true` if the access rights are `Some` and [`is_addable`](AccessRights::is_addable)
    /// is `true` for them.
    pub fn is_addable(self) -> bool {
        self.1.is_addable()
    }

    /// Formats the address and access rights of the [`URef`] in an unique way that could be used as
    /// a name when storing the given `URef` in a global state.
    pub fn as_string(&self) -> String {
        // Extract bits as numerical value, with no flags marked as 0.
        let access_rights_bits = self.access_rights().bits();
        // Access rights is represented as octal, which means that max value of u8 can
        // be represented as maximum of 3 octal digits.
        format!(
            "uref-{}-{:03o}",
            base16::encode_lower(&self.addr()),
            access_rights_bits
        )
    }
}

impl Display for URef {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let addr = self.addr();
        let access_rights = self.access_rights();
        write!(f, "URef({}, {})", HexFmt(&addr), access_rights)
    }
}

impl Debug for URef {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl TryFrom<Key> for URef {
    type Error = ApiError;

    fn try_from(key: Key) -> Result<Self, Self::Error> {
        if let Key::URef(uref) = key {
            Ok(uref)
        } else {
            Err(ApiError::UnexpectedKeyVariant)
        }
    }
}

impl Default for URef {
    fn default() -> Self {
        URef([0; UREF_ADDR_LENGTH], AccessRights::NONE)
    }
}

impl Serialize for URef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("uref", 2)?;
        state.serialize_field("addr", serde_bytes::Bytes::new(self.0.as_ref()))?;
        state.serialize_field("rights", &self.1)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for URef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct URefVisitor;

        impl<'de> Visitor<'de> for URefVisitor {
            type Value = URef;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a serialized URef")
            }

            fn visit_seq<V: SeqAccess<'de>>(self, mut seq: V) -> Result<URef, V::Error> {
                let bytes: &[u8] = seq
                    .next_element()?
                    .ok_or_else(|| SerdeError::invalid_length(0, &self))?;
                if bytes.len() != UREF_ADDR_LENGTH {
                    return Err(SerdeError::invalid_length(bytes.len(), &"32"));
                }
                let mut addr = [0; UREF_ADDR_LENGTH];
                addr.copy_from_slice(bytes);

                let rights: AccessRights = seq
                    .next_element()?
                    .ok_or_else(|| SerdeError::invalid_length(1, &self))?;
                Ok(URef(addr, rights))
            }
        }

        deserializer.deserialize_struct("uref", &["addr", "rights"], URefVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::encoding;

    #[test]
    fn uref_as_string() {
        // Since we are putting URefs to named_keys map keyed by the label that
        // `as_string()` returns, any changes to the string representation of
        // that type cannot break the format.
        let addr_array = [0u8; 32];
        let uref_a = URef::new(addr_array, AccessRights::READ);
        assert_eq!(
            uref_a.as_string(),
            "uref-0000000000000000000000000000000000000000000000000000000000000000-001"
        );
        let uref_b = URef::new(addr_array, AccessRights::WRITE);
        assert_eq!(
            uref_b.as_string(),
            "uref-0000000000000000000000000000000000000000000000000000000000000000-002"
        );

        let uref_c = uref_b.remove_access_rights();
        assert_eq!(
            uref_c.as_string(),
            "uref-0000000000000000000000000000000000000000000000000000000000000000-000"
        );
    }

    #[test]
    fn serialized_length() {
        let uref = URef::new([0; 32], AccessRights::READ);
        let actual_length = encoding::serialized_length(&uref).unwrap();
        assert_eq!(actual_length as usize, UREF_SERIALIZED_LENGTH);
    }
}
