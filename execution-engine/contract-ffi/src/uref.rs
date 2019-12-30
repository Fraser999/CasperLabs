//! Home of [`URef`](crate::uref::URef), which represents an unforgeable reference.

// Can be removed once https://github.com/rust-lang/rustfmt/issues/3362 is resolved.
#[rustfmt::skip]
use alloc::vec;
use alloc::{format, string::String, vec::Vec};

use base16;
use bitflags::bitflags;
use hex_fmt::HexFmt;

use crate::{
    bytesrepr::{self, OPTION_TAG_SERIALIZED_LENGTH},
    contract_api::TURef,
    value::CLTyped,
};

pub const UREF_ADDR_LENGTH: usize = 32;
pub const ACCESS_RIGHTS_SERIALIZED_LENGTH: usize = 1;
pub const UREF_SERIALIZED_LENGTH: usize =
    UREF_ADDR_LENGTH + OPTION_TAG_SERIALIZED_LENGTH + ACCESS_RIGHTS_SERIALIZED_LENGTH;

bitflags! {
    #[allow(clippy::derive_hash_xor_eq)]
    pub struct AccessRights: u8 {
        const READ  = 0b001;
        const WRITE = 0b010;
        const ADD   = 0b100;
        const READ_ADD       = Self::READ.bits | Self::ADD.bits;
        const READ_WRITE     = Self::READ.bits | Self::WRITE.bits;
        const ADD_WRITE      = Self::ADD.bits  | Self::WRITE.bits;
        const READ_ADD_WRITE = Self::READ.bits | Self::ADD.bits | Self::WRITE.bits;
    }
}

impl AccessRights {
    pub fn is_readable(self) -> bool {
        self & AccessRights::READ == AccessRights::READ
    }

    pub fn is_writeable(self) -> bool {
        self & AccessRights::WRITE == AccessRights::WRITE
    }

    pub fn is_addable(self) -> bool {
        self & AccessRights::ADD == AccessRights::ADD
    }
}

impl core::fmt::Display for AccessRights {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
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

impl bytesrepr::ToBytes for AccessRights {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        self.bits.to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.bits.serialized_length()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        vec![]
    }
}

impl bytesrepr::FromBytes for AccessRights {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (id, rem): (u8, &[u8]) = bytesrepr::FromBytes::from_bytes(bytes)?;
        match AccessRights::from_bits(id) {
            Some(rights) => Ok((rights, rem)),
            None => Err(bytesrepr::Error::FormattingError),
        }
    }
}

/// Represents an unforgeable reference
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct URef([u8; UREF_ADDR_LENGTH], Option<AccessRights>);

impl core::fmt::Display for URef {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let addr = self.addr();
        let access_rights_o = self.access_rights();
        if let Some(access_rights) = access_rights_o {
            write!(f, "URef({}, {})", HexFmt(&addr), access_rights)
        } else {
            write!(f, "URef({}, None)", HexFmt(&addr))
        }
    }
}

impl core::fmt::Debug for URef {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl URef {
    /// Creates a [`URef`] from an id and access rights.
    pub fn new(id: [u8; UREF_ADDR_LENGTH], access_rights: AccessRights) -> Self {
        URef(id, Some(access_rights))
    }

    /// Creates a [`URef`] from an id and optional access rights.  [`URef::new`]
    /// is the preferred constructor for most common use-cases.
    #[cfg(any(test, feature = "gens"))]
    pub(crate) fn unsafe_new(
        id: [u8; UREF_ADDR_LENGTH],
        maybe_access_rights: Option<AccessRights>,
    ) -> Self {
        URef(id, maybe_access_rights)
    }

    /// Returns the address of this URef.
    pub fn addr(&self) -> [u8; UREF_ADDR_LENGTH] {
        self.0
    }

    /// Returns the access rights of this URef.
    pub fn access_rights(&self) -> Option<AccessRights> {
        self.1
    }

    /// Returns a new URef with updated access rights.
    pub fn with_access_rights(self, access_rights: AccessRights) -> Self {
        URef(self.0, Some(access_rights))
    }

    /// Removes the access rights from this URef.
    pub fn remove_access_rights(self) -> Self {
        URef(self.0, None)
    }

    pub fn is_readable(self) -> bool {
        if let Some(access_rights) = self.1 {
            access_rights.is_readable()
        } else {
            false
        }
    }

    pub fn into_read(self) -> URef {
        URef(self.0, Some(AccessRights::READ))
    }

    pub fn into_read_add_write(self) -> URef {
        URef(self.0, Some(AccessRights::READ_ADD_WRITE))
    }

    pub fn is_writeable(self) -> bool {
        if let Some(access_rights) = self.1 {
            access_rights.is_writeable()
        } else {
            false
        }
    }

    pub fn is_addable(self) -> bool {
        if let Some(access_rights) = self.1 {
            access_rights.is_addable()
        } else {
            false
        }
    }

    /// Formats address and its access rights in an unique way that could be
    /// used as a name when storing given uref in a global state.
    pub fn as_string(&self) -> String {
        // Extract bits as numerical value, with no flags marked as 0.
        let access_rights_bits = self
            .access_rights()
            .map(|value| value.bits())
            .unwrap_or_default();
        // Access rights is represented as octal, which means that max value of u8 can
        // be represented as maximum of 3 octal digits.
        format!(
            "uref-{}-{:03o}",
            base16::encode_lower(&self.addr()),
            access_rights_bits
        )
    }
}

impl bytesrepr::ToBytes for URef {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = Vec::with_capacity(UREF_SERIALIZED_LENGTH);
        result.append(&mut self.0.to_bytes()?);
        result.append(&mut self.1.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length() + self.1.serialized_length()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        vec![0]
    }
}

impl bytesrepr::FromBytes for URef {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (id, rem): ([u8; 32], &[u8]) = bytesrepr::FromBytes::from_bytes(bytes)?;
        let (maybe_access_rights, rem): (Option<AccessRights>, &[u8]) =
            bytesrepr::FromBytes::from_bytes(rem)?;
        Ok((URef(id, maybe_access_rights), rem))
    }
}

impl<T: CLTyped> From<TURef<T>> for URef {
    fn from(input: TURef<T>) -> Self {
        URef(input.addr(), Some(input.access_rights()))
    }
}

#[allow(clippy::unnecessary_operation)]
#[cfg(test)]
mod tests {
    use crate::uref::{AccessRights, URef};

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
}
