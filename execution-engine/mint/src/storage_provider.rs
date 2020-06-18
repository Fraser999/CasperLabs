use serde::{de::DeserializeOwned, Serialize};

use types::{system_contract_errors::mint::Error, CLTyped, URef};

pub trait StorageProvider {
    fn new_uref<T: CLTyped + Serialize>(&mut self, init: T) -> URef;

    fn write_local<K: Serialize, V: CLTyped + Serialize>(&mut self, key: K, value: V);

    fn read_local<K: Serialize, V: CLTyped + DeserializeOwned>(
        &mut self,
        key: &K,
    ) -> Result<Option<V>, Error>;

    fn read<T: CLTyped + DeserializeOwned>(&mut self, uref: URef) -> Result<Option<T>, Error>;

    fn write<T: CLTyped + Serialize>(&mut self, uref: URef, value: T) -> Result<(), Error>;

    fn add<T: CLTyped + Serialize>(&mut self, uref: URef, value: T) -> Result<(), Error>;
}
