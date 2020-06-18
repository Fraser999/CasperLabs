use serde::{Deserialize, Serialize};

/// The number of bytes in a serialized [`BlockTime`].
pub const BLOCKTIME_SERIALIZED_LENGTH: usize = 8;

/// A newtype wrapping a [`u64`] which represents the block time.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct BlockTime(u64);

impl BlockTime {
    /// Constructs a `BlockTime`.
    pub fn new(value: u64) -> Self {
        BlockTime(value)
    }

    /// Saturating integer subtraction. Computes `self - other`, saturating at `0` instead of
    /// overflowing.
    pub fn saturating_sub(self, other: BlockTime) -> Self {
        BlockTime(self.0.saturating_sub(other.0))
    }
}

impl Into<u64> for BlockTime {
    fn into(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::encoding;

    #[test]
    fn serialized_length() {
        let actual_length = encoding::serialized_length(&BlockTime::new(0)).unwrap();
        assert_eq!(actual_length as usize, BLOCKTIME_SERIALIZED_LENGTH);
    }
}
