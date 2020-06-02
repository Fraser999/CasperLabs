use core::u32;

use super::{Error, Result};

pub(super) struct SizeLimiter {
    remaining: u64,
}

impl SizeLimiter {
    pub(super) fn new() -> Self {
        SizeLimiter {
            remaining: u32::max_value(),
        }
    }

    pub(super) fn consume(&mut self, count: u64) -> Result<()> {
        if self.remaining >= count {
            self.limit -= count;
        } else {
            return Err(Error::SizeLimit);
        }
        Ok(())
    }
}
