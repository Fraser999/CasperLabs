use std::collections::BTreeMap;

use contract_ffi::{
    bytesrepr::{Error, FromBytes, ToBytes, U32_SERIALIZED_LENGTH, U64_SERIALIZED_LENGTH},
    key::{Key, KEY_UREF_SERIALIZED_LENGTH},
    value::ProtocolVersion,
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Contract {
    bytes: Vec<u8>,
    named_keys: BTreeMap<String, Key>,
    protocol_version: ProtocolVersion,
}

impl Contract {
    pub fn new(
        bytes: Vec<u8>,
        named_keys: BTreeMap<String, Key>,
        protocol_version: ProtocolVersion,
    ) -> Self {
        Contract {
            bytes,
            named_keys,
            protocol_version,
        }
    }

    pub fn named_keys_append(&mut self, keys: &mut BTreeMap<String, Key>) {
        self.named_keys.append(keys);
    }

    pub fn named_keys(&self) -> &BTreeMap<String, Key> {
        &self.named_keys
    }

    pub fn named_keys_mut(&mut self) -> &mut BTreeMap<String, Key> {
        &mut self.named_keys
    }

    pub fn destructure(self) -> (Vec<u8>, BTreeMap<String, Key>, ProtocolVersion) {
        (self.bytes, self.named_keys, self.protocol_version)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    pub fn take_named_keys(self) -> BTreeMap<String, Key> {
        self.named_keys
    }
}

impl ToBytes for Contract {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        if self.bytes.len()
            + KEY_UREF_SERIALIZED_LENGTH * self.named_keys.len()
            + U64_SERIALIZED_LENGTH
            >= u32::max_value() as usize - U32_SERIALIZED_LENGTH * 2
        {
            return Err(Error::OutOfMemoryError);
        }
        let size: usize = U32_SERIALIZED_LENGTH +                        //size for length of bytes
                    self.bytes.len() +                                   //size for elements of bytes
                    U32_SERIALIZED_LENGTH +                              //size for length of named_keys
                    KEY_UREF_SERIALIZED_LENGTH * self.named_keys.len() + //size for named_keys elements
                    U64_SERIALIZED_LENGTH; //size for protocol_version

        let mut result = Vec::with_capacity(size);
        result.append(&mut self.bytes.to_bytes()?);
        result.append(&mut self.named_keys.to_bytes()?);
        result.append(&mut self.protocol_version.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.bytes.serialized_length()
            + self.named_keys.serialized_length()
            + self.protocol_version.serialized_length()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        // We probably don't need to actually calculate the offsets here since a serialized Contract
        // shouldn't be getting passed from the client to the host, and hence shouldn't be used as a
        // means of discovering new URefs.  So we could likely just return `vec![]` here without any
        // negative effects.
        let mut result = vec![];
        let running_offset = self.bytes.serialized_length() as u32;

        for offset in self.named_keys.uref_offsets() {
            result.push(running_offset + offset);
        }

        result
    }
}

impl FromBytes for Contract {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (bytes, rem1): (Vec<u8>, &[u8]) = FromBytes::from_bytes(bytes)?;
        let (named_keys, rem2): (BTreeMap<String, Key>, &[u8]) = FromBytes::from_bytes(rem1)?;
        let (protocol_version, rem3): (ProtocolVersion, &[u8]) = FromBytes::from_bytes(rem2)?;
        Ok((
            Contract {
                bytes,
                named_keys,
                protocol_version,
            },
            rem3,
        ))
    }
}

pub mod gens {
    use proptest::{collection::vec, prelude::*};

    use contract_ffi::gens::{named_keys_arb, protocol_version_arb};

    use super::Contract;

    pub fn contract_arb() -> impl Strategy<Value = Contract> {
        protocol_version_arb().prop_flat_map(move |protocol_version_arb| {
            named_keys_arb(20).prop_flat_map(move |urefs| {
                vec(any::<u8>(), 1..1000)
                    .prop_map(move |body| Contract::new(body, urefs.clone(), protocol_version_arb))
            })
        })
    }
}
