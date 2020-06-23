//! Some newtypes.
mod macros;

use std::{
    array::TryFromSliceError,
    convert::TryFrom,
    fmt::{self, Debug, Display, Formatter, LowerHex, UpperHex},
};

use blake2::{
    digest::{Input, VariableOutput},
    VarBlake2b,
};
use serde::{
    de::{Error as SerdeError, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use uuid::Uuid;

pub use types::BLAKE2B_DIGEST_LENGTH;

/// Represents a 32-byte BLAKE2b hash digest
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blake2bHash([u8; BLAKE2B_DIGEST_LENGTH]);

impl Blake2bHash {
    /// Creates a 32-byte BLAKE2b hash digest from a given a piece of data
    #[inline]
    pub fn new(data: &[u8]) -> Self {
        let mut ret = [0u8; BLAKE2B_DIGEST_LENGTH];
        // Safe to unwrap here because our digest length is constant and valid
        let mut hasher = VarBlake2b::new(BLAKE2B_DIGEST_LENGTH).unwrap();
        hasher.input(data);
        hasher.variable_result(|hash| ret.clone_from_slice(hash));
        Blake2bHash(ret)
    }

    /// Returns the underlying BLKAE2b hash bytes
    #[inline]
    pub fn value(&self) -> [u8; BLAKE2B_DIGEST_LENGTH] {
        self.0
    }

    /// Converts the underlying BLAKE2b hash digest array to a `Vec`
    #[inline]
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl LowerHex for Blake2bHash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let hex_string = base16::encode_lower(&self.value());
        if f.alternate() {
            write!(f, "0x{}", hex_string)
        } else {
            write!(f, "{}", hex_string)
        }
    }
}

impl UpperHex for Blake2bHash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let hex_string = base16::encode_upper(&self.value());
        if f.alternate() {
            write!(f, "0x{}", hex_string)
        } else {
            write!(f, "{}", hex_string)
        }
    }
}

impl Display for Blake2bHash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Blake2bHash({:#x})", self)
    }
}

impl Debug for Blake2bHash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<[u8; BLAKE2B_DIGEST_LENGTH]> for Blake2bHash {
    fn from(arr: [u8; BLAKE2B_DIGEST_LENGTH]) -> Self {
        Blake2bHash(arr)
    }
}

impl<'a> TryFrom<&'a [u8]> for Blake2bHash {
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &[u8]) -> Result<Blake2bHash, Self::Error> {
        <[u8; BLAKE2B_DIGEST_LENGTH]>::try_from(slice).map(Blake2bHash)
    }
}

impl From<Blake2bHash> for [u8; BLAKE2B_DIGEST_LENGTH] {
    fn from(hash: Blake2bHash) -> Self {
        hash.0
    }
}

impl Serialize for Blake2bHash {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0.as_ref())
    }
}

impl<'de> Deserialize<'de> for Blake2bHash {
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BytesVisitor;

        impl<'de> Visitor<'de> for BytesVisitor {
            type Value = &'de [u8];

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a borrowed byte array")
            }

            #[inline]
            fn visit_borrowed_bytes<E: serde::de::Error>(
                self,
                bytes: &'de [u8],
            ) -> Result<Self::Value, E> {
                Ok(bytes)
            }
        }

        let bytes = deserializer.deserialize_bytes(BytesVisitor)?;
        if bytes.len() != BLAKE2B_DIGEST_LENGTH {
            return Err(SerdeError::invalid_length(bytes.len(), &"32"));
        }
        let mut array = [0; BLAKE2B_DIGEST_LENGTH];
        array.copy_from_slice(bytes);
        Ok(Blake2bHash(array))
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Serialize)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    pub fn new() -> CorrelationId {
        CorrelationId(Uuid::new_v4())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_nil()
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        newtypes::{Blake2bHash, CorrelationId},
        utils,
    };
    use std::hash::{Hash, Hasher};

    #[test]
    fn should_be_able_to_generate_correlation_id() {
        let correlation_id = CorrelationId::new();

        //println!("{}", correlation_id.to_string());

        assert_ne!(
            correlation_id.to_string(),
            "00000000-0000-0000-0000-000000000000",
            "should not be empty value"
        )
    }

    #[test]
    fn should_support_to_string() {
        let correlation_id = CorrelationId::new();

        assert!(
            !correlation_id.is_empty(),
            "correlation_id should be produce string"
        )
    }

    #[test]
    fn should_support_to_string_no_type_encasement() {
        let correlation_id = CorrelationId::new();

        let correlation_id_string = correlation_id.to_string();

        assert!(
            !correlation_id_string.starts_with("CorrelationId"),
            "correlation_id should just be the inner value without tuple name"
        )
    }

    #[test]
    fn should_support_to_json() {
        let correlation_id = CorrelationId::new();

        let correlation_id_json = utils::jsonify(correlation_id, false);

        assert!(
            !correlation_id_json.is_empty(),
            "correlation_id should be produce json"
        )
    }

    #[test]
    fn should_support_is_display() {
        let correlation_id = CorrelationId::new();

        let display = format!("{}", correlation_id);

        assert!(!display.is_empty(), "display should not be empty")
    }

    #[test]
    fn should_support_is_empty() {
        let correlation_id = CorrelationId::new();

        assert!(
            !correlation_id.is_empty(),
            "correlation_id should not be empty"
        )
    }

    #[test]
    fn should_create_unique_id_on_new() {
        let correlation_id_lhs = CorrelationId::new();
        let correlation_id_rhs = CorrelationId::new();

        assert_ne!(
            correlation_id_lhs, correlation_id_rhs,
            "correlation_ids should be distinct"
        );
    }

    #[test]
    fn should_support_clone() {
        let correlation_id = CorrelationId::new();

        let cloned = correlation_id;

        assert_eq!(correlation_id, cloned, "should be cloneable")
    }

    #[test]
    fn should_support_copy() {
        let correlation_id = CorrelationId::new();

        let cloned = correlation_id;

        assert_eq!(correlation_id, cloned, "should be cloneable")
    }

    #[test]
    fn should_support_hash() {
        let correlation_id = CorrelationId::new();

        let mut state = std::collections::hash_map::DefaultHasher::new();

        correlation_id.hash(&mut state);

        let hash = state.finish();

        assert!(hash > 0, "should be hashable");
    }

    #[test]
    fn should_display_blake2bhash_in_hex() {
        let hash = Blake2bHash([0u8; 32]);
        let hash_hex = format!("{}", hash);
        assert_eq!(
            hash_hex,
            "Blake2bHash(0x0000000000000000000000000000000000000000000000000000000000000000)"
        );
    }

    #[test]
    fn should_print_blake2bhash_lower_hex() {
        let hash = Blake2bHash([10u8; 32]);
        let hash_lower_hex = format!("{:x}", hash);
        assert_eq!(
            hash_lower_hex,
            "0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a"
        )
    }

    #[test]
    fn should_print_blake2bhash_upper_hex() {
        let hash = Blake2bHash([10u8; 32]);
        let hash_lower_hex = format!("{:X}", hash);
        assert_eq!(
            hash_lower_hex,
            "0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A"
        )
    }

    #[test]
    fn alternate_should_prepend_0x() {
        let hash = Blake2bHash([0u8; 32]);
        let hash_hex_alt = format!("{:#x}", hash);
        assert_eq!(
            hash_hex_alt,
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
    }
}
