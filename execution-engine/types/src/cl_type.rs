use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::{Key, URef, U128, U256, U512};

/// CasperLabs types, i.e. types which can be stored and manipulated by smart contracts.
///
/// Provides a description of the underlying data type of a [`CLValue`](crate::CLValue).
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum CLType {
    /// `bool` primitive.
    Bool,
    /// `i32` primitive.
    I32,
    /// `i64` primitive.
    I64,
    /// `u8` primitive.
    U8,
    /// `u32` primitive.
    U32,
    /// `u64` primitive.
    U64,
    /// [`U128`] large unsigned integer type.
    U128,
    /// [`U256`] large unsigned integer type.
    U256,
    /// [`U512`] large unsigned integer type.
    U512,
    /// `()` primitive.
    Unit,
    /// `String` primitive.
    String,
    /// [`Key`] system type.
    Key,
    /// [`URef`] system type.
    URef,
    /// `Option` of a `CLType`.
    Option(Box<CLType>),
    /// Variable-length list of a single `CLType` (comparable to a `Vec`).
    List(Box<CLType>),
    /// Fixed-length list of a single `CLType` (comparable to a Rust array).
    FixedList(Box<CLType>, u32),
    /// `Result` with `Ok` and `Err` variants of `CLType`s.
    #[allow(missing_docs)] // generated docs are explicit enough.
    Result { ok: Box<CLType>, err: Box<CLType> },
    /// Map with keys of a single `CLType` and values of a single `CLType`.
    #[allow(missing_docs)] // generated docs are explicit enough.
    Map {
        key: Box<CLType>,
        value: Box<CLType>,
    },
    /// 1-ary tuple of a `CLType`.
    Tuple1([Box<CLType>; 1]),
    /// 2-ary tuple of `CLType`s.
    Tuple2([Box<CLType>; 2]),
    /// 3-ary tuple of `CLType`s.
    Tuple3([Box<CLType>; 3]),
    /// Unspecified type.
    Any,
}

/// Returns the `CLType` describing a "named key" on the system, i.e. a `(String, Key)`.
pub fn named_key_type() -> CLType {
    CLType::Tuple2([Box::new(CLType::String), Box::new(CLType::Key)])
}

/// A type which can be described as a [`CLType`].
pub trait CLTyped {
    /// The `CLType` of `Self`.
    fn cl_type() -> CLType;
}

impl CLTyped for bool {
    fn cl_type() -> CLType {
        CLType::Bool
    }
}

impl CLTyped for i32 {
    fn cl_type() -> CLType {
        CLType::I32
    }
}

impl CLTyped for i64 {
    fn cl_type() -> CLType {
        CLType::I64
    }
}

impl CLTyped for u8 {
    fn cl_type() -> CLType {
        CLType::U8
    }
}

impl CLTyped for u32 {
    fn cl_type() -> CLType {
        CLType::U32
    }
}

impl CLTyped for u64 {
    fn cl_type() -> CLType {
        CLType::U64
    }
}

impl CLTyped for U128 {
    fn cl_type() -> CLType {
        CLType::U128
    }
}

impl CLTyped for U256 {
    fn cl_type() -> CLType {
        CLType::U256
    }
}

impl CLTyped for U512 {
    fn cl_type() -> CLType {
        CLType::U512
    }
}

impl CLTyped for () {
    fn cl_type() -> CLType {
        CLType::Unit
    }
}

impl CLTyped for String {
    fn cl_type() -> CLType {
        CLType::String
    }
}

impl CLTyped for &str {
    fn cl_type() -> CLType {
        CLType::String
    }
}

impl CLTyped for Key {
    fn cl_type() -> CLType {
        CLType::Key
    }
}

impl CLTyped for URef {
    fn cl_type() -> CLType {
        CLType::URef
    }
}

impl<T: CLTyped> CLTyped for Option<T> {
    fn cl_type() -> CLType {
        CLType::Option(Box::new(T::cl_type()))
    }
}

impl<T: CLTyped> CLTyped for Vec<T> {
    fn cl_type() -> CLType {
        CLType::List(Box::new(T::cl_type()))
    }
}

macro_rules! impl_cl_typed_for_array {
    ($($N:literal)+) => {
        $(
            impl<T: CLTyped> CLTyped for [T; $N] {
                fn cl_type() -> CLType {
                    CLType::FixedList(Box::new(T::cl_type()), $N as u32)
                }
            }
        )+
    }
}

impl_cl_typed_for_array! {
      0  1  2  3  4  5  6  7  8  9
     10 11 12 13 14 15 16 17 18 19
     20 21 22 23 24 25 26 27 28 29
     30 31 32
     64 128 256 512
}

impl<T: CLTyped, E: CLTyped> CLTyped for Result<T, E> {
    fn cl_type() -> CLType {
        let ok = Box::new(T::cl_type());
        let err = Box::new(E::cl_type());
        CLType::Result { ok, err }
    }
}

impl<K: CLTyped, V: CLTyped> CLTyped for BTreeMap<K, V> {
    fn cl_type() -> CLType {
        let key = Box::new(K::cl_type());
        let value = Box::new(V::cl_type());
        CLType::Map { key, value }
    }
}

impl<T1: CLTyped> CLTyped for (T1,) {
    fn cl_type() -> CLType {
        CLType::Tuple1([Box::new(T1::cl_type())])
    }
}

impl<T1: CLTyped, T2: CLTyped> CLTyped for (T1, T2) {
    fn cl_type() -> CLType {
        CLType::Tuple2([Box::new(T1::cl_type()), Box::new(T2::cl_type())])
    }
}

impl<T1: CLTyped, T2: CLTyped, T3: CLTyped> CLTyped for (T1, T2, T3) {
    fn cl_type() -> CLType {
        CLType::Tuple3([
            Box::new(T1::cl_type()),
            Box::new(T2::cl_type()),
            Box::new(T3::cl_type()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, string::ToString};

    use serde::{de::DeserializeOwned, Serialize};
    use serde_big_array::big_array;

    use super::*;
    use crate::{encoding, AccessRights, CLValue};

    fn round_trip<T: CLTyped + Serialize + DeserializeOwned + PartialEq + Debug + Clone>(
        value: &T,
    ) {
        let cl_value = CLValue::from_t(value.clone()).unwrap();

        let serialized_cl_value = encoding::serialize(&cl_value).unwrap();
        assert_eq!(
            serialized_cl_value.len(),
            encoding::serialized_length(&cl_value).unwrap() as usize
        );
        let parsed_cl_value: CLValue = encoding::deserialize(&serialized_cl_value).unwrap();
        assert_eq!(cl_value, parsed_cl_value);

        let parsed_value = CLValue::into_t(cl_value).unwrap();
        assert_eq!(*value, parsed_value);
    }

    #[test]
    fn bool_should_work() {
        round_trip(&true);
        round_trip(&false);
    }

    #[test]
    fn u8_should_work() {
        round_trip(&1u8);
    }

    #[test]
    fn u32_should_work() {
        round_trip(&1u32);
    }

    #[test]
    fn i32_should_work() {
        round_trip(&-1i32);
    }

    #[test]
    fn u64_should_work() {
        round_trip(&1u64);
    }

    #[test]
    fn i64_should_work() {
        round_trip(&-1i64);
    }

    #[test]
    fn u128_should_work() {
        round_trip(&U128::one());
    }

    #[test]
    fn u256_should_work() {
        round_trip(&U256::one());
    }

    #[test]
    fn u512_should_work() {
        round_trip(&U512::one());
    }

    #[test]
    fn unit_should_work() {
        round_trip(&());
    }

    #[test]
    fn string_should_work() {
        round_trip(&String::from("abc"));
    }

    #[test]
    fn key_should_work() {
        let key = Key::URef(URef::new([0u8; 32], AccessRights::READ_ADD_WRITE));
        round_trip(&key);
    }

    #[test]
    fn uref_should_work() {
        let uref = URef::new([0u8; 32], AccessRights::READ_ADD_WRITE);
        round_trip(&uref);
    }

    #[test]
    fn option_of_cl_type_should_work() {
        let x: Option<i32> = Some(-1);
        let y: Option<i32> = None;

        round_trip(&x);
        round_trip(&y);
    }

    #[test]
    fn vec_of_cl_type_should_work() {
        let vec = vec![String::from("a"), String::from("b")];
        round_trip(&vec);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn small_array_of_cl_type_should_work() {
        macro_rules! test_small_array {
            ($($N:literal)+) => {
                $(
                    let mut array = [0u64; $N];
                    for i in 0..$N {
                        array[i] = i as u64;
                    }
                    round_trip(&array);
                )+
            }
        }

        test_small_array! {
              0  1  2  3  4  5  6  7  8  9
             10 11 12 13 14 15 16 17 18 19
             20 21 22 23 24 25 26 27 28 29
             30 31 32
        }
    }

    #[test]
    fn large_array_of_cl_type_should_work() {
        big_array! { BigArray; }

        macro_rules! test_large_array {
            ($($N:literal $name:ident)+) => {
                $(
                    #[derive(Clone, Serialize, Deserialize)]
                    struct $name(#[serde(with = "BigArray")] [u64; $N]);

                    impl CLTyped for $name {
                        fn cl_type() -> CLType {
                            CLType::FixedList(Box::new(CLType::U64), $N as u32)
                        }
                    }

                    let array = {
                        let mut tmp = [0u64; $N];
                        for i in 0..$N {
                            tmp[i] = i as u64;
                        }
                        $name(tmp)
                    };

                    let cl_value = CLValue::from_t(array.clone()).unwrap();

                    let serialized_cl_value = encoding::serialize(&cl_value).unwrap();
                    let parsed_cl_value: CLValue = encoding::deserialize(&serialized_cl_value).unwrap();
                    assert_eq!(cl_value, parsed_cl_value);

                    let parsed_value: $name = CLValue::into_t(cl_value).unwrap();
                    for i in 0..$N {
                        assert_eq!(array.0[i], parsed_value.0[i]);
                    }
                )+
            }
        }

        test_large_array! { 64 Array64 128 Array128 256 Array256 512 Array512 }
    }

    #[test]
    fn result_of_cl_type_should_work() {
        let x: Result<(), String> = Ok(());
        let y: Result<(), String> = Err(String::from("Hello, world!"));

        round_trip(&x);
        round_trip(&y);
    }

    #[test]
    fn map_of_cl_type_should_work() {
        let mut map: BTreeMap<String, u64> = BTreeMap::new();
        map.insert(String::from("abc"), 1);
        map.insert(String::from("xyz"), 2);

        round_trip(&map);
    }

    #[test]
    fn tuple_1_should_work() {
        let x = (-1i32,);

        round_trip(&x);
    }

    #[test]
    fn tuple_2_should_work() {
        let x = (-1i32, String::from("a"));

        round_trip(&x);
    }

    #[test]
    fn tuple_3_should_work() {
        let x = (-1i32, 1u32, String::from("a"));

        round_trip(&x);
    }

    #[test]
    fn any_should_work() {
        #[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
        struct Any(String);

        impl CLTyped for Any {
            fn cl_type() -> CLType {
                CLType::Any
            }
        }

        let any = Any("Any test".to_string());
        round_trip(&any);
    }
}
