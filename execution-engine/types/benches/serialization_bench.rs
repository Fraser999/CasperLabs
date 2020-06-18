#![feature(test)]

extern crate test;

use std::{collections::BTreeMap, iter};

use serde::{de::DeserializeOwned, Serialize};
use serde_bytes::ByteBuf;
use test::{black_box, Bencher};

use casperlabs_types::{
    account::PublicKey, encoding, AccessRights, CLTyped, CLValue, Key, URef, U128, U256, U512,
};

static KB: usize = 1024;
static BATCH: usize = 4 * KB;

const TEST_I32: i32 = 123_456_789;
const TEST_U128: U128 = U128([123_456_789, 0]);
const TEST_U256: U256 = U256([123_456_789, 0, 0, 0]);
const TEST_U512: U512 = U512([123_456_789, 0, 0, 0, 0, 0, 0, 0]);
const TEST_STR_1: &str = "String One";
const TEST_STR_2: &str = "String Two";

fn prepare_vector(size: usize) -> Vec<i32> {
    (0..size as i32).collect()
}

#[bench]
fn serialize_vector_of_i32s(b: &mut Bencher) {
    let data = prepare_vector(black_box(BATCH));
    b.iter(|| encoding::serialize(&data));
}

#[bench]
fn deserialize_vector_of_i32s(b: &mut Bencher) {
    let data = encoding::serialize(&prepare_vector(black_box(BATCH))).unwrap();
    b.iter(|| {
        encoding::deserialize::<Vec<i32>>(&data).unwrap();
    });
}

#[bench]
fn serialize_vector_of_u8(b: &mut Bencher) {
    // 0, 1, ... 254, 255, 0, 1, ...
    let data = prepare_vector(BATCH)
        .into_iter()
        .map(|value| value as u8)
        .collect::<Vec<u8>>();
    let data = ByteBuf::from(data);
    b.iter(|| encoding::serialize(&data));
}

#[bench]
fn deserialize_vector_of_u8(b: &mut Bencher) {
    // 0, 1, ... 254, 255, 0, 1, ...
    let data: Vec<u8> = encoding::serialize(
        &prepare_vector(BATCH)
            .into_iter()
            .map(|value| value as u8)
            .collect::<Vec<_>>(),
    )
    .unwrap();
    b.iter(|| encoding::deserialize::<ByteBuf>(&data))
}

#[bench]
fn serialize_u8(b: &mut Bencher) {
    b.iter(|| encoding::serialize(&black_box(&129u8)));
}

#[bench]
fn deserialize_u8(b: &mut Bencher) {
    b.iter(|| encoding::deserialize::<u8>(black_box(&[129u8])));
}

#[bench]
fn serialize_i32(b: &mut Bencher) {
    b.iter(|| encoding::serialize(&black_box(&1_816_142_132i32)));
}

#[bench]
fn deserialize_i32(b: &mut Bencher) {
    b.iter(|| encoding::deserialize::<i32>(black_box(&[0x34, 0x21, 0x40, 0x6c])));
}

#[bench]
fn serialize_u64(b: &mut Bencher) {
    b.iter(|| encoding::serialize(&black_box(&14_157_907_845_468_752_670u64)));
}

#[bench]
fn deserialize_u64(b: &mut Bencher) {
    b.iter(|| {
        encoding::deserialize::<u64>(black_box(&[0x1e, 0x8b, 0xe1, 0x73, 0x2c, 0xfe, 0x7a, 0xc4]))
    });
}

#[bench]
fn serialize_some_u64(b: &mut Bencher) {
    let data = Some(14_157_907_845_468_752_670u64);

    b.iter(|| encoding::serialize(&black_box(&data)));
}

#[bench]
fn deserialize_some_u64(b: &mut Bencher) {
    let data = Some(14_157_907_845_468_752_670u64);
    let data = encoding::serialize(&data).unwrap();

    b.iter(|| encoding::deserialize::<u64>(&data));
}

#[bench]
fn serialize_none_u64(b: &mut Bencher) {
    let data: Option<u64> = None;

    b.iter(|| encoding::serialize(&black_box(&data)));
}

#[bench]
fn deserialize_ok_u64(b: &mut Bencher) {
    let data: Option<u64> = None;
    let data = encoding::serialize(&data).unwrap();
    b.iter(|| encoding::deserialize::<Option<u64>>(&data));
}

#[bench]
fn serialize_vector_of_vector_of_u8(b: &mut Bencher) {
    let data: Vec<ByteBuf> = (0..4)
        .map(|_v| {
            // 0, 1, 2, ..., 254, 255
            let v = iter::repeat_with(|| 0..255u8)
                .flatten()
                // 4 times to create 4x 1024 bytes
                .take(4)
                .collect::<Vec<u8>>();
            ByteBuf::from(v)
        })
        .collect();

    b.iter(|| encoding::serialize(&data));
}

#[bench]
fn deserialize_vector_of_vector_of_u8(b: &mut Bencher) {
    let data: Vec<u8> = encoding::serialize(
        &(0..4)
            .map(|_v| {
                // 0, 1, 2, ..., 254, 255
                iter::repeat_with(|| 0..255u8)
                    .flatten()
                    // 4 times to create 4x 1024 bytes
                    .take(4)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<Vec<_>>>(),
    )
    .unwrap();
    b.iter(|| encoding::deserialize::<Vec<ByteBuf>>(&data));
}

#[bench]
fn serialize_tree_map(b: &mut Bencher) {
    let data = {
        let mut res = BTreeMap::new();
        res.insert("asdf".to_string(), "zxcv".to_string());
        res.insert("qwer".to_string(), "rewq".to_string());
        res.insert("1234".to_string(), "5678".to_string());
        res
    };

    b.iter(|| encoding::serialize(&black_box(&data)));
}

#[bench]
fn deserialize_treemap(b: &mut Bencher) {
    let data = {
        let mut res = BTreeMap::new();
        res.insert("asdf".to_string(), "zxcv".to_string());
        res.insert("qwer".to_string(), "rewq".to_string());
        res.insert("1234".to_string(), "5678".to_string());
        res
    };
    let data = encoding::serialize(&data).unwrap();
    b.iter(|| encoding::deserialize::<BTreeMap<String, String>>(black_box(&data)));
}

#[bench]
fn serialize_string(b: &mut Bencher) {
    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
    let data = lorem.to_string();
    b.iter(|| encoding::serialize(black_box(&data)));
}

#[bench]
fn deserialize_string(b: &mut Bencher) {
    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
    let data = encoding::serialize(&lorem).unwrap();
    b.iter(|| encoding::deserialize::<String>(&data));
}

#[bench]
fn serialize_vec_of_string(b: &mut Bencher) {
    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string();
    let array_of_lorem: Vec<String> = lorem.split(' ').map(Into::into).collect();
    let data = array_of_lorem;
    b.iter(|| encoding::serialize(black_box(&data)));
}

#[bench]
fn deserialize_vec_of_string(b: &mut Bencher) {
    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string();
    let array_of_lorem: Vec<String> = lorem.split(' ').map(Into::into).collect();
    let data = encoding::serialize(&array_of_lorem).unwrap();

    b.iter(|| encoding::deserialize::<Vec<String>>(&data));
}

#[bench]
fn serialize_unit(b: &mut Bencher) {
    b.iter(|| encoding::serialize(black_box(&())))
}

#[bench]
fn deserialize_unit(b: &mut Bencher) {
    let data = encoding::serialize(&()).unwrap();

    b.iter(|| encoding::deserialize::<()>(&data))
}

#[bench]
fn serialize_key_account(b: &mut Bencher) {
    let account = Key::Account(PublicKey::ed25519_from([0u8; 32]));

    b.iter(|| encoding::serialize(black_box(&account)))
}

#[bench]
fn deserialize_key_account(b: &mut Bencher) {
    let account = Key::Account(PublicKey::ed25519_from([0u8; 32]));
    let account_bytes = encoding::serialize(&account).unwrap();

    b.iter(|| encoding::deserialize::<Key>(black_box(&account_bytes)))
}

#[bench]
fn serialize_key_hash(b: &mut Bencher) {
    let hash = Key::Hash([0u8; 32]);
    b.iter(|| encoding::serialize(black_box(&hash)))
}

#[bench]
fn deserialize_key_hash(b: &mut Bencher) {
    let hash = Key::Hash([0u8; 32]);
    let hash_bytes = encoding::serialize(&hash).unwrap();

    b.iter(|| encoding::deserialize::<Key>(black_box(&hash_bytes)))
}

#[bench]
fn serialize_key_uref(b: &mut Bencher) {
    let uref = Key::URef(URef::new([0u8; 32], AccessRights::ADD_WRITE));
    b.iter(|| encoding::serialize(black_box(&uref)))
}

#[bench]
fn deserialize_key_uref(b: &mut Bencher) {
    let uref = Key::URef(URef::new([0u8; 32], AccessRights::ADD_WRITE));
    let uref_bytes = encoding::serialize(&uref).unwrap();

    b.iter(|| encoding::deserialize::<Key>(black_box(&uref_bytes)))
}

#[bench]
fn serialize_vec_of_keys(b: &mut Bencher) {
    let keys: Vec<Key> = (0..32)
        .map(|i| Key::URef(URef::new([i; 32], AccessRights::ADD_WRITE)))
        .collect();
    b.iter(|| encoding::serialize(black_box(&keys)))
}

#[bench]
fn deserialize_vec_of_keys(b: &mut Bencher) {
    let keys: Vec<Key> = (0..32)
        .map(|i| Key::URef(URef::new([i; 32], AccessRights::ADD_WRITE)))
        .collect();
    let keys_bytes = encoding::serialize(&keys).unwrap();
    b.iter(|| encoding::deserialize::<Vec<Key>>(black_box(&keys_bytes)));
}

#[bench]
fn serialize_access_rights_read(b: &mut Bencher) {
    b.iter(|| encoding::serialize(&AccessRights::READ));
}

#[bench]
fn deserialize_access_rights_read(b: &mut Bencher) {
    let data = encoding::serialize(&AccessRights::READ).unwrap();
    b.iter(|| encoding::deserialize::<AccessRights>(&data));
}

#[bench]
fn serialize_access_rights_write(b: &mut Bencher) {
    b.iter(|| encoding::serialize(&AccessRights::WRITE));
}

#[bench]
fn deserialize_access_rights_write(b: &mut Bencher) {
    let data = encoding::serialize(&AccessRights::WRITE).unwrap();
    b.iter(|| encoding::deserialize::<AccessRights>(&data));
}

#[bench]
fn serialize_access_rights_add(b: &mut Bencher) {
    b.iter(|| encoding::serialize(&AccessRights::ADD));
}

#[bench]
fn deserialize_access_rights_add(b: &mut Bencher) {
    let data = encoding::serialize(&AccessRights::ADD).unwrap();
    b.iter(|| encoding::deserialize::<AccessRights>(&data));
}

#[bench]
fn serialize_access_rights_read_add(b: &mut Bencher) {
    b.iter(|| encoding::serialize(&AccessRights::READ_ADD));
}

#[bench]
fn deserialize_access_rights_read_add(b: &mut Bencher) {
    let data = encoding::serialize(&AccessRights::READ_ADD).unwrap();
    b.iter(|| encoding::deserialize::<AccessRights>(&data));
}

#[bench]
fn serialize_access_rights_read_write(b: &mut Bencher) {
    b.iter(|| encoding::serialize(&AccessRights::READ_WRITE));
}

#[bench]
fn deserialize_access_rights_read_write(b: &mut Bencher) {
    let data = encoding::serialize(&AccessRights::READ_WRITE).unwrap();
    b.iter(|| encoding::deserialize::<AccessRights>(&data));
}

#[bench]
fn serialize_access_rights_add_write(b: &mut Bencher) {
    b.iter(|| encoding::serialize(&AccessRights::ADD_WRITE));
}

#[bench]
fn deserialize_access_rights_add_write(b: &mut Bencher) {
    let data = encoding::serialize(&AccessRights::ADD_WRITE).unwrap();
    b.iter(|| encoding::deserialize::<AccessRights>(&data));
}

fn serialize_cl_value<T: CLTyped + Serialize>(raw_value: T) -> Vec<u8> {
    encoding::serialize(&CLValue::from_t(raw_value).expect("should create CLValue"))
        .expect("should serialize CLValue")
}

fn benchmark_deserialization<T: CLTyped + Serialize + DeserializeOwned>(
    b: &mut Bencher,
    raw_value: T,
) {
    let serialized_value = serialize_cl_value(raw_value);
    b.iter(|| {
        let cl_value: CLValue = encoding::deserialize(&serialized_value).unwrap();
        let _raw_value: T = cl_value.into_t().unwrap();
    });
}

#[bench]
fn serialize_cl_value_int32(b: &mut Bencher) {
    b.iter(|| serialize_cl_value(TEST_I32));
}

#[bench]
fn deserialize_cl_value_int32(b: &mut Bencher) {
    benchmark_deserialization(b, TEST_I32);
}

#[bench]
fn serialize_cl_value_uint128(b: &mut Bencher) {
    b.iter(|| serialize_cl_value(TEST_U128));
}

#[bench]
fn deserialize_cl_value_uint128(b: &mut Bencher) {
    benchmark_deserialization(b, TEST_U128);
}

#[bench]
fn serialize_cl_value_uint256(b: &mut Bencher) {
    b.iter(|| serialize_cl_value(TEST_U256));
}

#[bench]
fn deserialize_cl_value_uint256(b: &mut Bencher) {
    benchmark_deserialization(b, TEST_U256);
}

#[bench]
fn serialize_cl_value_uint512(b: &mut Bencher) {
    b.iter(|| serialize_cl_value(TEST_U512));
}

#[bench]
fn deserialize_cl_value_uint512(b: &mut Bencher) {
    benchmark_deserialization(b, TEST_U512);
}

#[bench]
fn serialize_cl_value_bytearray(b: &mut Bencher) {
    b.iter(|| serialize_cl_value((0..255).collect::<Vec<u8>>()));
}

#[bench]
fn deserialize_cl_value_bytearray(b: &mut Bencher) {
    benchmark_deserialization(b, (0..255).collect::<Vec<u8>>());
}

#[bench]
fn serialize_cl_value_listint32(b: &mut Bencher) {
    b.iter(|| serialize_cl_value((0..1024).collect::<Vec<i32>>()));
}

#[bench]
fn deserialize_cl_value_listint32(b: &mut Bencher) {
    benchmark_deserialization(b, (0..1024).collect::<Vec<i32>>());
}

#[bench]
fn serialize_cl_value_string(b: &mut Bencher) {
    b.iter(|| serialize_cl_value(TEST_STR_1.to_string()));
}

#[bench]
fn deserialize_cl_value_string(b: &mut Bencher) {
    benchmark_deserialization(b, TEST_STR_1.to_string());
}

#[bench]
fn serialize_cl_value_liststring(b: &mut Bencher) {
    b.iter(|| serialize_cl_value(vec![TEST_STR_1.to_string(), TEST_STR_2.to_string()]));
}

#[bench]
fn deserialize_cl_value_liststring(b: &mut Bencher) {
    benchmark_deserialization(b, vec![TEST_STR_1.to_string(), TEST_STR_2.to_string()]);
}

#[bench]
fn serialize_cl_value_namedkey(b: &mut Bencher) {
    b.iter(|| {
        serialize_cl_value((
            TEST_STR_1.to_string(),
            Key::Account(PublicKey::ed25519_from([0xffu8; 32])),
        ))
    });
}

#[bench]
fn deserialize_cl_value_namedkey(b: &mut Bencher) {
    benchmark_deserialization(
        b,
        (
            TEST_STR_1.to_string(),
            Key::Account(PublicKey::ed25519_from([0xffu8; 32])),
        ),
    );
}

#[bench]
fn serialize_u128(b: &mut Bencher) {
    let num_u128 = U128::default();
    b.iter(|| encoding::serialize(black_box(&num_u128)))
}

#[bench]
fn deserialize_u128(b: &mut Bencher) {
    let num_u128 = U128::default();
    let num_u128_bytes = encoding::serialize(&num_u128).unwrap();

    b.iter(|| encoding::deserialize::<U128>(black_box(&num_u128_bytes)))
}

#[bench]
fn serialize_u256(b: &mut Bencher) {
    let num_u256 = U256::default();
    b.iter(|| encoding::serialize(black_box(&num_u256)))
}

#[bench]
fn deserialize_u256(b: &mut Bencher) {
    let num_u256 = U256::default();
    let num_u256_bytes = encoding::serialize(&num_u256).unwrap();

    b.iter(|| encoding::deserialize::<U256>(black_box(&num_u256_bytes)))
}

#[bench]
fn serialize_u512(b: &mut Bencher) {
    let num_u512 = U512::default();
    b.iter(|| encoding::serialize(black_box(&num_u512)))
}

#[bench]
fn deserialize_u512(b: &mut Bencher) {
    let num_u512 = U512::default();
    let num_u512_bytes = encoding::serialize(&num_u512).unwrap();

    b.iter(|| encoding::deserialize::<U512>(black_box(&num_u512_bytes)))
}
