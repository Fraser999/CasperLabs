use super::*;

use engine_shared::stored_value::StoredValue;
use types::{account::PublicKey, encoding, CLValue, Key};

#[test]
fn radix_is_256() {
    assert_eq!(
        super::RADIX,
        256,
        "Changing RADIX alone might cause things to break"
    );
}

#[test]
fn round_trip() {
    let leaf = Trie::Leaf {
        key: Key::Account(PublicKey::ed25519_from([0; 32])),
        value: StoredValue::CLValue(CLValue::from_t(42_i32).unwrap()),
    };
    encoding::test_serialization_roundtrip(&leaf);

    let node = Trie::<Key, StoredValue>::Node {
        pointer_block: Box::new(PointerBlock::default()),
    };
    encoding::test_serialization_roundtrip(&node);

    let extension = Trie::<Key, StoredValue>::Extension {
        affix: (0..255).collect(),
        pointer: Pointer::NodePointer(Blake2bHash::new(&[0; 32])),
    };
    encoding::test_serialization_roundtrip(&extension);
}

mod pointer_block {
    use engine_shared::newtypes::Blake2bHash;

    use crate::trie::*;

    /// A defense against changes to [`RADIX`](history::trie::RADIX).
    #[test]
    fn debug_formatter_succeeds() {
        let _ = format!("{:?}", PointerBlock::new());
    }

    #[test]
    fn assignment_and_indexing() {
        let test_hash = Blake2bHash::new(b"TrieTrieAgain");
        let leaf_pointer = Some(Pointer::LeafPointer(test_hash));
        let mut pointer_block = PointerBlock::new();
        pointer_block[0] = leaf_pointer;
        pointer_block[RADIX - 1] = leaf_pointer;
        assert_eq!(leaf_pointer, pointer_block[0]);
        assert_eq!(leaf_pointer, pointer_block[RADIX - 1]);
        assert_eq!(None, pointer_block[1]);
        assert_eq!(None, pointer_block[RADIX - 2]);
    }

    #[test]
    #[should_panic]
    fn assignment_off_end() {
        let test_hash = Blake2bHash::new(b"TrieTrieAgain");
        let leaf_pointer = Some(Pointer::LeafPointer(test_hash));
        let mut pointer_block = PointerBlock::new();
        pointer_block[RADIX] = leaf_pointer;
    }

    #[test]
    #[should_panic]
    fn indexing_off_end() {
        let pointer_block = PointerBlock::new();
        let _val = pointer_block[RADIX];
    }
}

mod proptests {
    use proptest::prelude::proptest;

    use types::encoding;

    use crate::trie::gens::*;

    proptest! {
        #[test]
        fn roundtrip_blake2b_hash(hash in blake2b_hash_arb()) {
            encoding::test_serialization_roundtrip(&hash);
        }

        #[test]
        fn roundtrip_trie_pointer(pointer in trie_pointer_arb()) {
            encoding::test_serialization_roundtrip(&pointer);
        }

        #[test]
        fn roundtrip_trie_pointer_block(pointer_block in trie_pointer_block_arb()) {
            encoding::test_serialization_roundtrip(&pointer_block);
        }

        #[test]
        fn roundtrip_trie(trie in trie_arb()) {
            encoding::test_serialization_roundtrip(&trie);
        }
    }
}
