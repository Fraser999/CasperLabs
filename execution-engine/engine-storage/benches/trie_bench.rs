#![feature(test)]

extern crate test;

use test::Bencher;

use casperlabs_engine_storage::trie::{Pointer, PointerBlock, Trie};
use engine_shared::{newtypes::Blake2bHash, stored_value::StoredValue};
use types::{account::PublicKey, encoding, CLValue, Key};

#[bench]
fn serialize_trie_leaf(b: &mut Bencher) {
    let leaf = Trie::Leaf {
        key: Key::Account(PublicKey::ed25519_from([0; 32])),
        value: StoredValue::CLValue(CLValue::from_t(42_i32).unwrap()),
    };
    b.iter(|| encoding::serialize(&leaf));
}

#[bench]
fn deserialize_trie_leaf(b: &mut Bencher) {
    let leaf = Trie::Leaf {
        key: Key::Account(PublicKey::ed25519_from([0; 32])),
        value: StoredValue::CLValue(CLValue::from_t(42_i32).unwrap()),
    };
    let leaf_bytes = encoding::serialize(&leaf).unwrap();
    b.iter(|| encoding::deserialize::<Trie<Key, StoredValue>>(&leaf_bytes));
}

#[bench]
fn serialize_trie_node(b: &mut Bencher) {
    let node = Trie::<Key, StoredValue>::Node {
        pointer_block: Box::new(PointerBlock::default()),
    };
    b.iter(|| encoding::serialize(&node));
}

#[bench]
fn deserialize_trie_node(b: &mut Bencher) {
    let node = Trie::<Key, StoredValue>::Node {
        pointer_block: Box::new(PointerBlock::default()),
    };
    let node_bytes = encoding::serialize(&node).unwrap();

    b.iter(|| encoding::deserialize::<Trie<Key, StoredValue>>(&node_bytes));
}

#[bench]
fn serialize_trie_node_pointer(b: &mut Bencher) {
    let extension = Trie::<Key, StoredValue>::Extension {
        affix: (0..255).collect(),
        pointer: Pointer::NodePointer(Blake2bHash::new(&[0; 32])),
    };

    b.iter(|| encoding::serialize(&extension));
}

#[bench]
fn deserialize_trie_node_pointer(b: &mut Bencher) {
    let extension = Trie::<Key, StoredValue>::Extension {
        affix: (0..255).collect(),
        pointer: Pointer::NodePointer(Blake2bHash::new(&[0; 32])),
    };
    let extension_bytes = encoding::serialize(&extension).unwrap();

    b.iter(|| encoding::deserialize::<Trie<Key, StoredValue>>(&extension_bytes));
}
