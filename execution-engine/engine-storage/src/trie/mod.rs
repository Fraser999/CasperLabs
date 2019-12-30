//! Core types for a Merkle Trie

use std::ops::Deref;

use contract_ffi::bytesrepr::{self, FromBytes, ToBytes, U32_SERIALIZED_LENGTH};
use engine_shared::newtypes::{Blake2bHash, BLAKE2B_DIGEST_LENGTH};

#[cfg(test)]
pub mod gens;

#[cfg(test)]
mod tests;

pub const RADIX: usize = 256;

/// A parent is represented as a pair of a child index and a node or extension.
pub type Parents<K, V> = Vec<(u8, Trie<K, V>)>;

/// Represents a pointer to the next object in a Merkle Trie
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

    fn tag(&self) -> u32 {
        match self {
            Pointer::LeafPointer(_) => 0,
            Pointer::NodePointer(_) => 1,
        }
    }
}

impl ToBytes for Pointer {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut hash_bytes = self.hash().to_bytes()?;
        let mut ret = Vec::with_capacity(U32_SERIALIZED_LENGTH + hash_bytes.len());
        ret.append(&mut self.tag().to_bytes()?);
        ret.append(&mut hash_bytes);
        Ok(ret)
    }

    fn serialized_length(&self) -> usize {
        U32_SERIALIZED_LENGTH + BLAKE2B_DIGEST_LENGTH
    }
}

impl FromBytes for Pointer {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (tag, rem): (u32, &[u8]) = FromBytes::from_bytes(bytes)?;
        match tag {
            0 => {
                let (hash, rem): (Blake2bHash, &[u8]) = FromBytes::from_bytes(rem)?;
                Ok((Pointer::LeafPointer(hash), rem))
            }
            1 => {
                let (hash, rem): (Blake2bHash, &[u8]) = FromBytes::from_bytes(rem)?;
                Ok((Pointer::NodePointer(hash), rem))
            }
            _ => Err(bytesrepr::Error::FormattingError),
        }
    }
}

/// Represents the underlying structure of a node in a Merkle Trie
#[derive(Copy, Clone)]
pub struct PointerBlock([Option<Pointer>; RADIX]);

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

impl ToBytes for PointerBlock {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        self.0.to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
    }
}

impl FromBytes for PointerBlock {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        FromBytes::from_bytes(bytes).map(|(arr, rem)| (PointerBlock(arr), rem))
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trie<K, V> {
    Leaf { key: K, value: V },
    Node { pointer_block: Box<PointerBlock> },
    Extension { affix: Vec<u8>, pointer: Pointer },
}

impl<K, V> Trie<K, V> {
    fn tag(&self) -> u32 {
        match self {
            Trie::Leaf { .. } => 0,
            Trie::Node { .. } => 1,
            Trie::Extension { .. } => 2,
        }
    }

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

impl<K, V> ToBytes for Trie<K, V>
where
    K: ToBytes,
    V: ToBytes,
{
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        match self {
            Trie::Leaf { key, value } => {
                let mut key_bytes = ToBytes::to_bytes(key)?;
                let mut value_bytes = ToBytes::to_bytes(value)?;
                if key_bytes.len() + value_bytes.len()
                    > u32::max_value() as usize - U32_SERIALIZED_LENGTH
                {
                    return Err(bytesrepr::Error::OutOfMemoryError);
                }
                let mut ret: Vec<u8> =
                    Vec::with_capacity(U32_SERIALIZED_LENGTH + key_bytes.len() + value_bytes.len());
                ret.append(&mut self.tag().to_bytes()?);
                ret.append(&mut key_bytes);
                ret.append(&mut value_bytes);
                Ok(ret)
            }
            Trie::Node { pointer_block } => {
                let mut pointer_block_bytes = ToBytes::to_bytes(pointer_block.deref())?;
                let mut ret: Vec<u8> =
                    Vec::with_capacity(U32_SERIALIZED_LENGTH + pointer_block_bytes.len());
                ret.append(&mut self.tag().to_bytes()?);
                ret.append(&mut pointer_block_bytes);
                Ok(ret)
            }
            Trie::Extension { affix, pointer } => {
                let mut affix_bytes = ToBytes::to_bytes(affix)?;
                let mut pointer_bytes = ToBytes::to_bytes(pointer)?;
                if affix_bytes.len() + pointer_bytes.len()
                    > u32::max_value() as usize - U32_SERIALIZED_LENGTH
                {
                    return Err(bytesrepr::Error::OutOfMemoryError);
                }
                let mut ret: Vec<u8> = Vec::with_capacity(
                    U32_SERIALIZED_LENGTH + affix_bytes.len() + pointer_bytes.len(),
                );
                ret.append(&mut self.tag().to_bytes()?);
                ret.append(&mut affix_bytes);
                ret.append(&mut pointer_bytes);
                Ok(ret)
            }
        }
    }

    fn serialized_length(&self) -> usize {
        U32_SERIALIZED_LENGTH
            + match self {
                Trie::Leaf { key, value } => key.serialized_length() + value.serialized_length(),
                Trie::Node { pointer_block } => pointer_block.serialized_length(),
                Trie::Extension { affix, pointer } => {
                    affix.serialized_length() + pointer.serialized_length()
                }
            }
    }
}

impl<K: FromBytes, V: FromBytes> FromBytes for Trie<K, V> {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (tag, rem): (u32, &[u8]) = FromBytes::from_bytes(bytes)?;
        match tag {
            0 => {
                let (key, rem): (K, &[u8]) = FromBytes::from_bytes(rem)?;
                let (value, rem): (V, &[u8]) = FromBytes::from_bytes(rem)?;
                Ok((Trie::Leaf { key, value }, rem))
            }
            1 => {
                let (pointer_block, rem): (PointerBlock, &[u8]) = FromBytes::from_bytes(rem)?;
                Ok((
                    Trie::Node {
                        pointer_block: Box::new(pointer_block),
                    },
                    rem,
                ))
            }
            2 => {
                let (affix, rem): (Vec<u8>, &[u8]) = FromBytes::from_bytes(rem)?;
                let (pointer, rem): (Pointer, &[u8]) = FromBytes::from_bytes(rem)?;
                Ok((Trie::Extension { affix, pointer }, rem))
            }
            _ => Err(bytesrepr::Error::FormattingError),
        }
    }
}

pub(crate) mod operations {
    use crate::trie::Trie;
    use contract_ffi::bytesrepr::{self, ToBytes};
    use engine_shared::newtypes::Blake2bHash;

    /// Creates a tuple containing an empty root hash and an empty root (a node
    /// with an empty pointer block)
    pub fn create_hashed_empty_trie<K: ToBytes, V: ToBytes>(
    ) -> Result<(Blake2bHash, Trie<K, V>), bytesrepr::Error> {
        let root: Trie<K, V> = Trie::Node {
            pointer_block: Default::default(),
        };
        let root_bytes: Vec<u8> = root.to_bytes()?;
        Ok((Blake2bHash::new(&root_bytes), root))
    }
}
