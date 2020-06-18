//! Core types for a Merkle Trie

use serde::{Deserialize, Serialize};

use engine_shared::newtypes::Blake2bHash;

#[cfg(test)]
pub mod gens;

#[cfg(test)]
mod tests;

pub const RADIX: usize = 256;

/// A parent is represented as a pair of a child index and a node or extension.
pub type Parents<K, V> = Vec<(u8, Trie<K, V>)>;

/// Represents a pointer to the next object in a Merkle Trie
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pointer {
    LeafPointer(Blake2bHash),
    NodePointer(Blake2bHash),
}

impl Pointer {
    pub fn hash(&self) -> &Blake2bHash {
        match self {
            Pointer::LeafPointer(hash) => hash,
            Pointer::NodePointer(hash) => hash,
        }
    }

    pub fn update(&self, hash: Blake2bHash) -> Self {
        match self {
            Pointer::LeafPointer(_) => Pointer::LeafPointer(hash),
            Pointer::NodePointer(_) => Pointer::NodePointer(hash),
        }
    }
}

// This is inside a private module so that the generated `BigArray` does not form part of this
// crate's public API, and hence also doesn't appear in the rustdocs.
mod big_array {
    use serde_big_array::big_array;

    big_array! { BigArray; }
}

/// Represents the underlying structure of a node in a Merkle Trie
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct PointerBlock(#[serde(with = "big_array::BigArray")] [Option<Pointer>; RADIX]);

impl PointerBlock {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_indexed_pointers(indexed_pointers: &[(usize, Pointer)]) -> Self {
        let mut ret = PointerBlock::new();
        for (idx, ptr) in indexed_pointers.iter() {
            ret[*idx] = Some(*ptr);
        }
        ret
    }
}

impl From<[Option<Pointer>; RADIX]> for PointerBlock {
    fn from(src: [Option<Pointer>; RADIX]) -> Self {
        PointerBlock(src)
    }
}

impl PartialEq for PointerBlock {
    #[inline]
    fn eq(&self, other: &PointerBlock) -> bool {
        self.0[..] == other.0[..]
    }
}

impl Eq for PointerBlock {}

impl Default for PointerBlock {
    fn default() -> Self {
        PointerBlock([Default::default(); RADIX])
    }
}

impl core::ops::Index<usize> for PointerBlock {
    type Output = Option<Pointer>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        let PointerBlock(dat) = self;
        &dat[index]
    }
}

impl core::ops::IndexMut<usize> for PointerBlock {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let PointerBlock(dat) = self;
        &mut dat[index]
    }
}

impl core::ops::Index<core::ops::Range<usize>> for PointerBlock {
    type Output = [Option<Pointer>];

    #[inline]
    fn index(&self, index: core::ops::Range<usize>) -> &[Option<Pointer>] {
        let &PointerBlock(ref dat) = self;
        &dat[index]
    }
}

impl core::ops::Index<core::ops::RangeTo<usize>> for PointerBlock {
    type Output = [Option<Pointer>];

    #[inline]
    fn index(&self, index: core::ops::RangeTo<usize>) -> &[Option<Pointer>] {
        let &PointerBlock(ref dat) = self;
        &dat[index]
    }
}

impl core::ops::Index<core::ops::RangeFrom<usize>> for PointerBlock {
    type Output = [Option<Pointer>];

    #[inline]
    fn index(&self, index: core::ops::RangeFrom<usize>) -> &[Option<Pointer>] {
        let &PointerBlock(ref dat) = self;
        &dat[index]
    }
}

impl core::ops::Index<core::ops::RangeFull> for PointerBlock {
    type Output = [Option<Pointer>];

    #[inline]
    fn index(&self, index: core::ops::RangeFull) -> &[Option<Pointer>] {
        let &PointerBlock(ref dat) = self;
        &dat[index]
    }
}

impl ::std::fmt::Debug for PointerBlock {
    #[allow(clippy::assertions_on_constants)]
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        assert!(RADIX > 1, "RADIX must be > 1");
        write!(f, "{}([", stringify!(PointerBlock))?;
        write!(f, "{:?}", self.0[0])?;
        for item in self.0[1..].iter() {
            write!(f, ", {:?}", item)?;
        }
        write!(f, "])")
    }
}

/// Represents a Merkle Trie
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trie<K, V> {
    Leaf { key: K, value: V },
    Node { pointer_block: Box<PointerBlock> },
    Extension { affix: Vec<u8>, pointer: Pointer },
}

impl<K, V> Trie<K, V> {
    /// Constructs a [`Trie::Leaf`] from a given key and value.
    pub fn leaf(key: K, value: V) -> Self {
        Trie::Leaf { key, value }
    }

    /// Constructs a [`Trie::Node`] from a given slice of indexed pointers.
    pub fn node(indexed_pointers: &[(usize, Pointer)]) -> Self {
        let pointer_block = PointerBlock::from_indexed_pointers(indexed_pointers);
        let pointer_block = Box::new(pointer_block);
        Trie::Node { pointer_block }
    }

    /// Constructs a [`Trie::Extension`] from a given affix and pointer.
    pub fn extension(affix: Vec<u8>, pointer: Pointer) -> Self {
        Trie::Extension { affix, pointer }
    }

    pub fn key(&self) -> Option<&K> {
        match self {
            Trie::Leaf { key, .. } => Some(key),
            _ => None,
        }
    }
}

pub(crate) mod operations {
    use serde::Serialize;

    use engine_shared::newtypes::Blake2bHash;
    use types::encoding;

    use crate::trie::Trie;

    /// Creates a tuple containing an empty root hash and an empty root (a node
    /// with an empty pointer block)
    pub fn create_hashed_empty_trie<K: Serialize, V: Serialize>(
    ) -> Result<(Blake2bHash, Trie<K, V>), encoding::Error> {
        let root: Trie<K, V> = Trie::Node {
            pointer_block: Default::default(),
        };
        let root_bytes: Vec<u8> = encoding::serialize(&root)?;
        Ok((Blake2bHash::new(&root_bytes), root))
    }
}
