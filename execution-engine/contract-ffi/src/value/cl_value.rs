use alloc::vec::Vec;

use crate::{
    bytesrepr::{self, FromBytes, ToBytes},
    uref::URef,
    value::cl_type::{CLType, CLTyped},
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CLTypeMismatch {
    pub expected: CLType,
    pub found: CLType,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum CLValueError {
    Serialization(bytesrepr::Error),
    Type(CLTypeMismatch),
}

impl From<bytesrepr::Error> for CLValueError {
    fn from(error: bytesrepr::Error) -> Self {
        CLValueError::Serialization(error)
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CLValue {
    cl_type: CLType,
    bytes: Vec<u8>,
    uref_offsets: Vec<u32>,
}

impl CLValue {
    /// Constructs a `CLValue` from `t`.
    pub fn from_t<T: CLTyped + ToBytes>(t: T) -> Result<CLValue, CLValueError> {
        let uref_offsets = t.uref_offsets();
        let bytes = t.into_bytes()?;

        Ok(CLValue {
            cl_type: T::cl_type(),
            bytes,
            uref_offsets,
        })
    }

    /// Consumes and converts `self` back into its underlying type.
    pub fn into_t<T: CLTyped + FromBytes>(self) -> Result<T, CLValueError> {
        let expected = T::cl_type();

        if self.cl_type == expected {
            Ok(bytesrepr::deserialize(self.bytes)?)
        } else {
            Err(CLValueError::Type(CLTypeMismatch {
                expected,
                found: self.cl_type,
            }))
        }
    }

    // This is only required in order to implement `TryFrom<state::CLValue> for CLValue` (i.e. the
    // conversion from the Protobuf `CLValue`) in a separate module to this one.
    #[doc(hidden)]
    pub fn from_components(cl_type: CLType, bytes: Vec<u8>, uref_offsets: Vec<u32>) -> Self {
        Self {
            cl_type,
            bytes,
            uref_offsets,
        }
    }

    // This is only required in order to implement `From<CLValue> for state::CLValue` (i.e. the
    // conversion to the Protobuf `CLValue`) in a separate module to this one.
    #[doc(hidden)]
    pub fn destructure(self) -> (CLType, Vec<u8>, Vec<u32>) {
        (self.cl_type, self.bytes, self.uref_offsets)
    }

    pub fn cl_type(&self) -> &CLType {
        &self.cl_type
    }

    /// Returns a reference to the serialized form of the underlying value held in this `CLValue`.
    pub fn inner_bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    pub fn contained_urefs(&self) -> Result<Vec<URef>, CLValueError> {
        let mut result = Vec::with_capacity(self.uref_offsets.len());
        for offset in &self.uref_offsets {
            let (_skipped_bytes, serialized_uref_with_remainder) =
                bytesrepr::safe_split_at(&self.bytes, *offset as usize)?;
            let (uref, _remainder) = URef::from_bytes(serialized_uref_with_remainder)?;
            result.push(uref);
        }
        Ok(result)
    }
}

impl ToBytes for CLValue {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        self.clone().into_bytes()
    }

    fn into_bytes(self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = self.bytes.into_bytes()?;
        self.cl_type.append_bytes(&mut result);
        result.append(&mut self.uref_offsets.into_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.bytes.serialized_length()
            + self.cl_type.serialized_length()
            + self.uref_offsets.serialized_length()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        self.uref_offsets.clone()
    }
}

impl FromBytes for CLValue {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (bytes, remainder) = Vec::<u8>::from_bytes(bytes)?;
        let (cl_type, remainder) = CLType::from_bytes(remainder)?;
        let (uref_offsets, remainder) = Vec::<u32>::from_bytes(remainder)?;
        let cl_value = CLValue {
            cl_type,
            bytes,
            uref_offsets,
        };
        Ok((cl_value, remainder))
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;

    use super::*;
    use crate::{
        bytesrepr::deserialize,
        uref::{AccessRights, URef},
    };

    #[test]
    fn should_serialize_cl_value() {
        let urefs = vec![
            URef::new([1; 32], AccessRights::ADD_WRITE),
            URef::new([2; 32], AccessRights::ADD_WRITE).remove_access_rights(),
            URef::new([3; 32], AccessRights::READ),
        ];

        let mut map = BTreeMap::new();
        map.insert(String::from("111"), urefs[0].clone());
        map.insert(String::from("2"), urefs[1].clone());
        map.insert(String::from("33333333333333"), urefs[2].clone());

        let cl_value = CLValue::from_t(map.clone()).unwrap();

        let recovered_urefs = cl_value.contained_urefs().unwrap();
        assert_eq!(urefs, recovered_urefs);

        let serialized_cl_value = cl_value.clone().into_bytes().unwrap();
        let parsed_cl_value = deserialize::<CLValue>(serialized_cl_value).unwrap();
        assert_eq!(cl_value, parsed_cl_value);

        let parsed_map = parsed_cl_value.into_t().unwrap();
        assert_eq!(map, parsed_map);
    }
}
