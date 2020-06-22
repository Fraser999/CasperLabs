#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

use casperlabs_engine_shared::newtypes::Blake2bHash;
use types::encoding;

#[bench]
fn serialize_blake_hash(b: &mut Bencher) {
    let hash = Blake2bHash::new(b"test");
    b.iter(|| black_box(encoding::serialize(&hash).unwrap()));
}

#[bench]
fn deserialize_blake_hash(b: &mut Bencher) {
    let hash = Blake2bHash::new(b"test");
    let serialized_hash = encoding::serialize(&hash).unwrap();

    b.iter(|| black_box(encoding::deserialize::<Blake2bHash>(&serialized_hash).unwrap()));
}
