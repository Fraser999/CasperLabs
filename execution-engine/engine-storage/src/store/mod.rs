mod store_ext;
#[cfg(test)]
pub(crate) mod tests;

use serde::{de::DeserializeOwned, Serialize};

use types::encoding;

pub use self::store_ext::StoreExt;
use crate::transaction_source::{Readable, Writable};

pub trait Store<K, V> {
    type Error: From<encoding::Error>;

    type Handle;

    fn handle(&self) -> Self::Handle;

    fn get<T>(&self, txn: &T, key: &K) -> Result<Option<V>, Self::Error>
    where
        T: Readable<Handle = Self::Handle>,
        K: Serialize,
        V: DeserializeOwned,
        Self::Error: From<T::Error>,
    {
        let handle = self.handle();
        match txn.read(handle, &encoding::serialize(key)?)? {
            None => Ok(None),
            Some(value_bytes) => {
                let value = encoding::deserialize(&value_bytes)?;
                Ok(Some(value))
            }
        }
    }

    fn put<T>(&self, txn: &mut T, key: &K, value: &V) -> Result<(), Self::Error>
    where
        T: Writable<Handle = Self::Handle>,
        K: Serialize,
        V: Serialize,
        Self::Error: From<T::Error>,
    {
        let handle = self.handle();
        txn.write(
            handle,
            &encoding::serialize(key)?,
            &encoding::serialize(value)?,
        )
        .map_err(Into::into)
    }
}
