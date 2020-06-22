use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use types::{Key, ProtocolVersion};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Contract {
    #[serde(with = "serde_bytes")]
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

pub mod gens {
    use proptest::{collection::vec, prelude::*};

    use types::gens::{named_keys_arb, protocol_version_arb};

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
