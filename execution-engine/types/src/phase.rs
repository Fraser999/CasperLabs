use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{
    de::{Error as SerdeError, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{CLType, CLTyped};

/// The number of bytes in a serialized [`Phase`].
pub const PHASE_SERIALIZED_LENGTH: usize = 1;

/// The phase in which a given contract is executing.
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum Phase {
    /// Set while committing the genesis or upgrade configurations.
    System = 0,
    /// Set while executing the payment code of a deploy.
    Payment = 1,
    /// Set while executing the session code of a deploy.
    Session = 2,
    /// Set while finalizing payment at the end of a deploy.
    FinalizePayment = 3,
}

impl Serialize for Phase {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.to_u8().expect("Phase is represented as a u8"))
    }
}

impl<'de> Deserialize<'de> for Phase {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u8::deserialize(deserializer)?;
        FromPrimitive::from_u8(value).ok_or_else(|| {
            D::Error::invalid_value(Unexpected::Unsigned(value as u64), &"valid phase value")
        })
    }
}

impl CLTyped for Phase {
    fn cl_type() -> CLType {
        CLType::U8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::encoding;

    #[test]
    fn serialized_length() {
        let actual_length = encoding::serialized_length(&Phase::Session).unwrap();
        assert_eq!(actual_length as usize, PHASE_SERIALIZED_LENGTH);
    }
}
