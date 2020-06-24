use alloc::{format, string::String};
use core::fmt::{self, Debug, Display, Formatter};

use blake2::{
    digest::{Input, VariableOutput},
    VarBlake2b,
};
use hex_fmt::HexFmt;
use serde::{
    de::{EnumAccess, Error as SerdeError, Unexpected, VariantAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_bytes::Bytes;

use crate::{account::PublicKey, ContractRef, URef};

/// The number of bytes in a Blake2b hash
pub const BLAKE2B_DIGEST_LENGTH: usize = 32;
/// The number of bytes in a [`Key::Hash`].
pub const KEY_HASH_LENGTH: usize = 32;
/// The number of bytes in a [`Key::Local`].
pub const KEY_LOCAL_LENGTH: usize = 64;
/// The number of bytes in the seed for a new [`Key::Local`].
pub const KEY_LOCAL_SEED_LENGTH: usize = 32;

/// Creates a 32-byte BLAKE2b hash digest from a given a piece of data
fn hash(bytes: &[u8]) -> [u8; BLAKE2B_DIGEST_LENGTH] {
    let mut ret = [0u8; BLAKE2B_DIGEST_LENGTH];
    // Safe to unwrap here because our digest length is constant and valid
    let mut hasher = VarBlake2b::new(BLAKE2B_DIGEST_LENGTH).unwrap();
    hasher.input(bytes);
    hasher.variable_result(|hash| ret.clone_from_slice(hash));
    ret
}

/// The type under which data (e.g. [`CLValue`](crate::CLValue)s, smart contracts, user accounts)
/// are indexed on the network.
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum Key {
    /// A `Key` under which a user account is stored.
    Account(PublicKey),
    /// A `Key` under which a smart contract is stored and which is the pseudo-hash of the
    /// contract.
    Hash([u8; KEY_HASH_LENGTH]),
    /// A `Key` which is a [`URef`], under which most types of data can be stored.
    URef(URef),
    /// A `Key` to data (normally a [`CLValue`](crate::CLValue)) which is held in local-storage
    /// rather than global-storage.
    Local {
        /// A value derived from the base key defining the local context.
        seed: [u8; KEY_LOCAL_SEED_LENGTH],
        /// A hash identifying the stored data.
        hash: [u8; BLAKE2B_DIGEST_LENGTH],
    },
}

impl Key {
    /// Constructs a new [`Key::Local`] by hashing `seed` concatenated with `key_bytes`.
    pub fn local(seed: [u8; KEY_LOCAL_SEED_LENGTH], key_bytes: &[u8]) -> Self {
        let key_hash = hash(&key_bytes);
        Key::Local {
            seed,
            hash: key_hash,
        }
    }

    // This method is not intended to be used by third party crates.
    #[doc(hidden)]
    pub fn type_string(&self) -> String {
        match self {
            Key::Account(_) => String::from("Key::Account"),
            Key::Hash(_) => String::from("Key::Hash"),
            Key::URef(_) => String::from("Key::URef"),
            Key::Local { .. } => String::from("Key::Local"),
        }
    }

    /// Returns the maximum size a [`Key`] can be serialized into.
    pub const fn max_serialized_length() -> usize {
        KEY_LOCAL_SEED_LENGTH + KEY_LOCAL_SEED_LENGTH + 9
    }

    /// If `self` is of type [`Key::URef`], returns `self` with the [`AccessRights`] stripped from
    /// the wrapped [`URef`], otherwise returns `self` unmodified.
    pub fn normalize(self) -> Key {
        match self {
            Key::URef(uref) => Key::URef(uref.remove_access_rights()),
            other => other,
        }
    }

    /// Returns a human-readable version of `self`, with the inner bytes encoded to Base16.
    pub fn as_string(&self) -> String {
        match self {
            Key::Account(PublicKey::Ed25519(addr)) => {
                format!("account-ed25519-{}", base16::encode_lower(&addr.value()))
            }
            Key::Hash(addr) => format!("hash-{}", base16::encode_lower(addr)),
            Key::URef(uref) => uref.as_string(),
            Key::Local { hash, .. } => format!("local-{}", base16::encode_lower(hash)),
        }
    }

    /// Consumes and converts `self` to a [`ContractRef`] if `self` is of type [`Key::Hash`] or
    /// [`Key::URef`], otherwise returns `None`.
    pub fn to_contract_ref(self) -> Option<ContractRef> {
        match self {
            Key::URef(uref) => Some(ContractRef::URef(uref)),
            Key::Hash(id) => Some(ContractRef::Hash(id)),
            _ => None,
        }
    }

    /// Returns the inner bytes of `self` if `self` is of type [`Key::Account`], otherwise returns
    /// `None`.
    pub fn into_account(self) -> Option<PublicKey> {
        match self {
            Key::Account(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Returns the inner bytes of `self` if `self` is of type [`Key::Hash`], otherwise returns
    /// `None`.
    pub fn into_hash(self) -> Option<[u8; KEY_HASH_LENGTH]> {
        match self {
            Key::Hash(hash) => Some(hash),
            _ => None,
        }
    }

    /// Returns a reference to the inner [`URef`] if `self` is of type [`Key::URef`], otherwise
    /// returns `None`.
    pub fn as_uref(&self) -> Option<&URef> {
        match self {
            Key::URef(uref) => Some(uref),
            _ => None,
        }
    }

    /// Returns the inner [`URef`] if `self` is of type [`Key::URef`], otherwise returns `None`.
    pub fn into_uref(self) -> Option<URef> {
        match self {
            Key::URef(uref) => Some(uref),
            _ => None,
        }
    }

    /// Returns the inner bytes of `self` if `self` is of type [`Key::Local`], otherwise returns
    /// `None`.
    pub fn into_local(self) -> Option<[u8; KEY_LOCAL_LENGTH]> {
        match self {
            Key::Local { seed, hash } => {
                let mut result = [0; KEY_LOCAL_LENGTH];
                result[..KEY_LOCAL_SEED_LENGTH].copy_from_slice(&seed);
                result[KEY_LOCAL_SEED_LENGTH..].copy_from_slice(&hash);
                Some(result)
            }
            _ => None,
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Key::Account(PublicKey::Ed25519(ed25519)) => write!(f, "Key::Account({})", ed25519),
            Key::Hash(addr) => write!(f, "Key::Hash({})", HexFmt(addr)),
            Key::URef(uref) => write!(f, "Key::{}", uref), /* Display impl for URef will append */
            // URef(…).
            Key::Local { seed, hash } => write!(f, "Key::Local({}{})", HexFmt(seed), HexFmt(hash)),
        }
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<URef> for Key {
    fn from(uref: URef) -> Key {
        Key::URef(uref)
    }
}

impl Default for Key {
    fn default() -> Self {
        Key::Hash(<[u8; 32]>::default())
    }
}

impl Serialize for Key {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Key::Account(public_key) => {
                serializer.serialize_newtype_variant("key", 0, "account", public_key)
            }
            Key::Hash(hash) => {
                serializer.serialize_newtype_variant("key", 1, "account", Bytes::new(hash.as_ref()))
            }
            Key::URef(uref) => serializer.serialize_newtype_variant("key", 2, "uref", uref),
            Key::Local { seed, hash } => serializer.serialize_newtype_variant(
                "key",
                3,
                "local",
                &(Bytes::new(seed.as_ref()), Bytes::new(hash.as_ref())),
            ),
        }
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeyVisitor;

        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = Key;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a serialized Key")
            }

            fn visit_enum<A: EnumAccess<'de>>(self, value: A) -> Result<Self::Value, A::Error> {
                let (variant_tag, value): (u8, _) = value.variant()?;
                match variant_tag {
                    0 => {
                        let public_key: PublicKey = value.newtype_variant()?;
                        Ok(Key::Account(public_key))
                    }
                    1 => {
                        let bytes: &[u8] = value.newtype_variant()?;
                        if bytes.len() != KEY_HASH_LENGTH {
                            return Err(SerdeError::invalid_length(bytes.len(), &"32"));
                        }
                        let mut array = [0; KEY_HASH_LENGTH];
                        array.copy_from_slice(bytes);
                        Ok(Key::Hash(array))
                    }
                    2 => {
                        let uref: URef = value.newtype_variant()?;
                        Ok(Key::URef(uref))
                    }
                    3 => {
                        let (seed_bytes, hash_bytes): (&[u8], &[u8]) = value.newtype_variant()?;

                        if seed_bytes.len() != KEY_LOCAL_SEED_LENGTH {
                            return Err(SerdeError::invalid_length(seed_bytes.len(), &"32"));
                        }
                        if hash_bytes.len() != KEY_LOCAL_SEED_LENGTH {
                            return Err(SerdeError::invalid_length(hash_bytes.len(), &"32"));
                        }

                        let mut seed = [0; KEY_LOCAL_SEED_LENGTH];
                        seed.copy_from_slice(seed_bytes);

                        let mut hash = [0; KEY_LOCAL_SEED_LENGTH];
                        hash.copy_from_slice(hash_bytes);

                        Ok(Key::Local { seed, hash })
                    }
                    _ => Err(SerdeError::invalid_value(
                        Unexpected::Unsigned(u64::from(variant_tag)),
                        &"a u8 in the range 0..4",
                    )),
                }
            }
        }

        deserializer.deserialize_enum("key", &["account", "hash", "uref", "local"], KeyVisitor)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::{encoding, AccessRights, URef};

    fn test_readable(right: AccessRights, is_true: bool) {
        assert_eq!(right.is_readable(), is_true)
    }

    #[test]
    fn serialization_roundtrip() {
        let addr_array = [255u8; 32];
        let public_key = PublicKey::ed25519_from(addr_array);

        let account_key = Key::Account(public_key);
        encoding::test_serialization_roundtrip(&account_key);

        let uref_key = Key::URef(URef::new(addr_array, AccessRights::READ));
        encoding::test_serialization_roundtrip(&uref_key);

        let hash_key = Key::Hash(addr_array);
        encoding::test_serialization_roundtrip(&hash_key);

        let local_key = Key::Local {
            seed: addr_array,
            hash: addr_array,
        };
        encoding::test_serialization_roundtrip(&local_key);
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
    fn should_display_key() {
        let expected_hash = core::iter::repeat("0").take(64).collect::<String>();
        let addr_array = [0u8; 32];
        let public_key = PublicKey::ed25519_from(addr_array);
        let account_key = Key::Account(public_key);
        assert_eq!(
            format!("{}", account_key),
            format!("Key::Account(Ed25519({}))", expected_hash)
        );
        let uref_key = Key::URef(URef::new(addr_array, AccessRights::READ));
        assert_eq!(
            format!("{}", uref_key),
            format!("Key::URef({}, READ)", expected_hash)
        );
        let hash_key = Key::Hash(addr_array);
        assert_eq!(
            format!("{}", hash_key),
            format!("Key::Hash({})", expected_hash)
        );
        let expected_hash = core::iter::repeat("0").take(128).collect::<String>();
        let local_key = Key::Local {
            seed: addr_array,
            hash: addr_array,
        };
        assert_eq!(
            format!("{}", local_key),
            format!("Key::Local({})", expected_hash)
        );
    }

    #[test]
    fn abuse_vec_key() {
        // Prefix is 2^32-1 = shouldn't allocate that much
        let bytes: Vec<u8> = vec![255, 255, 255, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let res = encoding::deserialize::<Vec<Key>>(&bytes);
        assert_eq!(res.expect_err("should fail"), encoding::Error::EndOfSlice);
    }

    #[test]
    fn check_key_account_getters() {
        let account = [42; 32];
        let public_key = PublicKey::ed25519_from(account);
        let key1 = Key::Account(public_key);
        assert_eq!(key1.into_account(), Some(public_key));
        assert!(key1.into_hash().is_none());
        assert!(key1.as_uref().is_none());
        assert!(key1.into_local().is_none());
    }

    #[test]
    fn check_key_hash_getters() {
        let hash = [42; KEY_HASH_LENGTH];
        let key1 = Key::Hash(hash);
        assert!(key1.into_account().is_none());
        assert_eq!(key1.into_hash(), Some(hash));
        assert!(key1.as_uref().is_none());
        assert!(key1.into_local().is_none());
    }

    #[test]
    fn check_key_uref_getters() {
        let uref = URef::new([42; 32], AccessRights::READ_ADD_WRITE);
        let key1 = Key::URef(uref);
        assert!(key1.into_account().is_none());
        assert!(key1.into_hash().is_none());
        assert_eq!(key1.as_uref(), Some(&uref));
        assert!(key1.into_local().is_none());
    }

    #[test]
    fn check_key_local_getters() {
        let local = [42; KEY_LOCAL_LENGTH];
        let key1 = Key::Local {
            seed: [42; KEY_LOCAL_SEED_LENGTH],
            hash: [42; BLAKE2B_DIGEST_LENGTH],
        };
        assert!(key1.into_account().is_none());
        assert!(key1.into_hash().is_none());
        assert!(key1.as_uref().is_none());
        assert_eq!(key1.into_local().map(|x| x.to_vec()), Some(local.to_vec()));
    }

    #[test]
    fn key_max_serialized_length() {
        let key_account = Key::Account(PublicKey::ed25519_from([42; 32]));
        assert!(
            (encoding::serialized_length(&key_account).unwrap() as usize)
                < Key::max_serialized_length()
        );

        let key_hash = Key::Hash([42; 32]);
        assert!(
            (encoding::serialized_length(&key_hash).unwrap() as usize)
                < Key::max_serialized_length()
        );

        let key_uref = Key::URef(URef::new([42; 32], AccessRights::READ));
        assert!(
            (encoding::serialized_length(&key_uref).unwrap() as usize)
                < Key::max_serialized_length()
        );

        let key_local = Key::local([42; 32], &[42; 32]);
        assert_eq!(
            encoding::serialized_length(&key_local).unwrap() as usize,
            Key::max_serialized_length()
        );
    }
}
