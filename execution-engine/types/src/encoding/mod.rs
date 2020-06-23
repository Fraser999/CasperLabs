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

    // TODO: Re-add this check. The equivalent is not used consistenly in the ToBytes/FromBytes
    //       implementation and it is skewing benchmarks negatively against serde.
    // deserializer.input_slice_is_empty()?;
    Ok(result)
}

/// Returns the size that an object would be if serialized.
pub fn serialized_length<T: serde::Serialize + ?Sized>(value: &T) -> Result<u32> {
    let mut size_checker = SizeChecker::new();

    value.serialize(&mut size_checker)?;
    Ok(size_checker.take_total())
}

// This test helper is not intended to be used by third party crates.
#[doc(hidden)]
/// Returns `true` if a we can serialize and then deserialize a value
pub fn test_serialization_roundtrip<T>(t: &T)
where
    T: PartialEq + serde::Serialize + serde::de::DeserializeOwned,
{
    let mut serialized = serialize(t).expect("Unable to serialize data");
    assert_eq!(
        serialized.len(),
        serialized_length(t).unwrap() as usize,
        "\nLength of serialized data: {},\nserialized_length() yielded: {},\nserialized data: {:?}",
        serialized.len(),
        serialized_length(t).unwrap(),
        serialized
    );
    let deserialized = deserialize(&serialized).unwrap();
    assert!(*t == deserialized);

    if !serialized.is_empty() {
        assert!(deserialize::<T>(&serialized[..serialized.len() - 1]).is_err());
    }
    serialized.push(0);
    assert!(deserialize::<T>(&serialized).is_err());
}

#[cfg(test)]
mod proptests {
    use proptest::{collection::vec, prelude::*};

    use crate::gens::*;

    proptest! {
        #[test]
        fn test_bool(u in any::<bool>()) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u8(u in any::<u8>()) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u16(u in any::<u16>()) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u32(u in any::<u32>()) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_i32(u in any::<i32>()) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u64(u in any::<u64>()) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_i64(u in any::<i64>()) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u8_slice_32(s in u8_slice_32()) {
            super::test_serialization_roundtrip(&s);
        }

        #[test]
        fn test_vec_u8(u in vec(any::<u8>(), 1..100)) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_vec_i32(u in vec(any::<i32>(), 1..100)) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_vec_vec_u8(u in vec(vec(any::<u8>(), 1..100), 10)) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_uref_map(m in named_keys_arb(20)) {
            super::test_serialization_roundtrip(&m);
        }

        #[test]
        fn test_array_u8_32(arr in any::<[u8; 32]>()) {
            super::test_serialization_roundtrip(&arr);
        }

        #[test]
        fn test_string(s in ".*") {
            super::test_serialization_roundtrip(&s);
        }

        #[test]
        fn test_option(o in proptest::option::of(key_arb())) {
            super::test_serialization_roundtrip(&o);
        }

        #[test]
        fn test_unit(unit in Just(())) {
            super::test_serialization_roundtrip(&unit);
        }

        #[test]
        fn test_u128_serialization(u in u128_arb()) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u256_serialization(u in u256_arb()) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u512_serialization(u in u512_arb()) {
            super::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_key_serialization(key in key_arb()) {
            super::test_serialization_roundtrip(&key);
        }

        #[test]
        fn test_cl_value_serialization(cl_value in cl_value_arb()) {
            super::test_serialization_roundtrip(&cl_value);
        }

        #[test]
        fn test_access_rights(access_right in access_rights_arb()) {
            super::test_serialization_roundtrip(&access_right);
        }

        #[test]
        fn test_uref(uref in uref_arb()) {
            super::test_serialization_roundtrip(&uref);
        }

        #[test]
        fn test_public_key(pk in public_key_arb()) {
            super::test_serialization_roundtrip(&pk);
        }

        #[test]
        fn test_result(result in result_arb()) {
            super::test_serialization_roundtrip(&result);
        }

        #[test]
        fn test_phase_serialization(phase in phase_arb()) {
            super::test_serialization_roundtrip(&phase);
        }

        #[test]
        fn test_protocol_version(protocol_version in protocol_version_arb()) {
            super::test_serialization_roundtrip(&protocol_version);
        }

        #[test]
        fn test_sem_ver(sem_ver in sem_ver_arb()) {
            super::test_serialization_roundtrip(&sem_ver);
        }

        #[test]
        fn test_tuple1(t in (any::<u8>(),)) {
            super::test_serialization_roundtrip(&t);
        }

        #[test]
        fn test_tuple2(t in (any::<u8>(),any::<u32>())) {
            super::test_serialization_roundtrip(&t);
        }

        #[test]
        fn test_tuple3(t in (any::<u8>(),any::<u32>(),any::<i32>())) {
            super::test_serialization_roundtrip(&t);
        }
    }
}
