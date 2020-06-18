use bitflags::bitflags;
use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};

bitflags! {
    /// A struct which behaves like a set of bitflags to define access rights associated with a
    /// [`URef`](crate::URef).
    #[allow(clippy::derive_hash_xor_eq)]
    pub struct AccessRights: u8 {
        /// No permissions
        const NONE = 0;
        /// Permission to read the value under the associated `URef`.
        const READ  = 0b001;
        /// Permission to write a value under the associated `URef`.
        const WRITE = 0b010;
        /// Permission to add to the value under the associated `URef`.
        const ADD   = 0b100;
        /// Permission to read or add to the value under the associated `URef`.
        const READ_ADD       = Self::READ.bits | Self::ADD.bits;
        /// Permission to read or write the value under the associated `URef`.
        const READ_WRITE     = Self::READ.bits | Self::WRITE.bits;
        /// Permission to add to, or write the value under the associated `URef`.
        const ADD_WRITE      = Self::ADD.bits  | Self::WRITE.bits;
        /// Permission to read, add to, or write the value under the associated `URef`.
        const READ_ADD_WRITE = Self::READ.bits | Self::ADD.bits | Self::WRITE.bits;
    }
}

impl AccessRights {
    /// Returns `true` if the `READ` flag is set.
    pub fn is_readable(self) -> bool {
        self & AccessRights::READ == AccessRights::READ
    }

    /// Returns `true` if the `WRITE` flag is set.
    pub fn is_writeable(self) -> bool {
        self & AccessRights::WRITE == AccessRights::WRITE
    }

    /// Returns `true` if the `ADD` flag is set.
    pub fn is_addable(self) -> bool {
        self & AccessRights::ADD == AccessRights::ADD
    }

    /// Returns `true` if no flags are set.
    pub fn is_none(self) -> bool {
        self == AccessRights::NONE
    }
}

impl Serialize for AccessRights {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.bits)
    }
}

impl<'de> Deserialize<'de> for AccessRights {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u8::deserialize(deserializer)?;
        AccessRights::from_bits(value).ok_or_else(|| {
            D::Error::invalid_value(
                Unexpected::Unsigned(value as u64),
                &"valid access rights value",
            )
        })
    }
}

impl core::fmt::Display for AccessRights {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            AccessRights::NONE => write!(f, "NONE"),
            AccessRights::READ => write!(f, "READ"),
            AccessRights::WRITE => write!(f, "WRITE"),
            AccessRights::ADD => write!(f, "ADD"),
            AccessRights::READ_ADD => write!(f, "READ_ADD"),
            AccessRights::READ_WRITE => write!(f, "READ_WRITE"),
            AccessRights::ADD_WRITE => write!(f, "ADD_WRITE"),
            AccessRights::READ_ADD_WRITE => write!(f, "READ_ADD_WRITE"),
            _ => write!(f, "UNKNOWN"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_readable(right: AccessRights, is_true: bool) {
        assert_eq!(right.is_readable(), is_true)
    }

    #[test]
    fn test_is_readable() {
        test_readable(AccessRights::READ, true);
        test_readable(AccessRights::READ_ADD, true);
        test_readable(AccessRights::READ_WRITE, true);
        test_readable(AccessRights::READ_ADD_WRITE, true);
        test_readable(AccessRights::ADD, false);
        test_readable(AccessRights::ADD_WRITE, false);
        test_readable(AccessRights::WRITE, false);
    }

    fn test_writable(right: AccessRights, is_true: bool) {
        assert_eq!(right.is_writeable(), is_true)
    }

    #[test]
    fn test_is_writable() {
        test_writable(AccessRights::WRITE, true);
        test_writable(AccessRights::READ_WRITE, true);
        test_writable(AccessRights::ADD_WRITE, true);
        test_writable(AccessRights::READ, false);
        test_writable(AccessRights::ADD, false);
        test_writable(AccessRights::READ_ADD, false);
        test_writable(AccessRights::READ_ADD_WRITE, true);
    }

    fn test_addable(right: AccessRights, is_true: bool) {
        assert_eq!(right.is_addable(), is_true)
    }

    #[test]
    fn test_is_addable() {
        test_addable(AccessRights::ADD, true);
        test_addable(AccessRights::READ_ADD, true);
        test_addable(AccessRights::READ_WRITE, false);
        test_addable(AccessRights::ADD_WRITE, true);
        test_addable(AccessRights::READ, false);
        test_addable(AccessRights::WRITE, false);
        test_addable(AccessRights::READ_ADD_WRITE, true);
    }
}
