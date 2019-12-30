//! Contains serialization and deserialization code for types used throughout the system.

// Can be removed once https://github.com/rust-lang/rustfmt/issues/3362 is resolved.
#[rustfmt::skip]
use alloc::vec;
use alloc::{
    collections::{BTreeMap, TryReserveError},
    string::String,
    vec::Vec,
};
use core::mem::{size_of, MaybeUninit};

use failure::Fail;

use crate::value::{ProtocolVersion, SemVer};

pub const I32_SERIALIZED_LENGTH: usize = size_of::<i32>();
pub const I64_SERIALIZED_LENGTH: usize = size_of::<i64>();
pub const U8_SERIALIZED_LENGTH: usize = size_of::<u8>();
pub const U16_SERIALIZED_LENGTH: usize = size_of::<u16>();
pub const U32_SERIALIZED_LENGTH: usize = size_of::<u32>();
pub const U64_SERIALIZED_LENGTH: usize = size_of::<u64>();
pub const U128_SERIALIZED_LENGTH: usize = size_of::<u128>();
pub const U256_SERIALIZED_LENGTH: usize = U128_SERIALIZED_LENGTH * 2;
pub const U512_SERIALIZED_LENGTH: usize = U256_SERIALIZED_LENGTH * 2;
pub const OPTION_TAG_SERIALIZED_LENGTH: usize = 1;
pub const SEM_VER_SERIALIZED_LENGTH: usize = 12;

pub trait ToBytes {
    fn to_bytes(&self) -> Result<Vec<u8>, Error>;
    fn into_bytes(self) -> Result<Vec<u8>, Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }
    fn serialized_length(&self) -> usize;
    fn uref_offsets(&self) -> Vec<u32>;
}

pub trait FromBytes: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error>;
    fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), Error> {
        Self::from_bytes(bytes.as_slice()).map(|(x, remainder)| (x, Vec::from(remainder)))
    }
}

#[derive(Debug, Fail, PartialEq, Eq, Clone)]
#[repr(u8)]
pub enum Error {
    #[fail(display = "Deserialization error: early end of stream")]
    EarlyEndOfStream = 0,

    #[fail(display = "Deserialization error: formatting error")]
    FormattingError,

    #[fail(display = "Deserialization error: left-over bytes")]
    LeftOverBytes,

    #[fail(display = "Serialization error: out of memory")]
    OutOfMemoryError,
}

impl From<TryReserveError> for Error {
    fn from(_: TryReserveError) -> Error {
        Error::OutOfMemoryError
    }
}

pub fn deserialize<T: FromBytes>(bytes: Vec<u8>) -> Result<T, Error> {
    let (t, remainder) = T::from_vec(bytes)?;
    if remainder.is_empty() {
        Ok(t)
    } else {
        Err(Error::LeftOverBytes)
    }
}

pub fn serialize(t: impl ToBytes) -> Result<Vec<u8>, Error> {
    t.into_bytes()
}

pub fn safe_split_at(bytes: &[u8], n: usize) -> Result<(&[u8], &[u8]), Error> {
    if n > bytes.len() {
        Err(Error::EarlyEndOfStream)
    } else {
        Ok(bytes.split_at(n))
    }
}

impl ToBytes for bool {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        u8::from(*self).to_bytes()
    }

    fn serialized_length(&self) -> usize {
        1
    }

    fn uref_offsets(&self) -> Vec<u32> {
        vec![]
    }
}

impl FromBytes for bool {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        match bytes.split_first() {
            None => Err(Error::EarlyEndOfStream),
            Some((byte, rem)) => match byte {
                1 => Ok((true, rem)),
                0 => Ok((false, rem)),
                _ => Err(Error::FormattingError),
            },
        }
    }
}

impl ToBytes for u8 {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        Ok(vec![*self])
    }

    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
    }

    fn uref_offsets(&self) -> Vec<u32> {
        vec![]
    }
}

impl FromBytes for u8 {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        match bytes.split_first() {
            None => Err(Error::EarlyEndOfStream),
            Some((byte, rem)) => Ok((*byte, rem)),
        }
    }
}

macro_rules! impl_to_from_bytes_for_integral {
    ($type:ty, $serialized_length:ident) => {
        impl ToBytes for $type {
            fn to_bytes(&self) -> Result<Vec<u8>, Error> {
                Ok(self.to_le_bytes().to_vec())
            }

            fn serialized_length(&self) -> usize {
                $serialized_length
            }

            fn uref_offsets(&self) -> Vec<u32> {
                vec![]
            }
        }

        impl FromBytes for $type {
            fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
                let mut result = [0u8; $serialized_length];
                let (bytes, remainder) = safe_split_at(bytes, $serialized_length)?;
                result.copy_from_slice(bytes);
                Ok((<$type>::from_le_bytes(result), remainder))
            }
        }
    };
}

impl_to_from_bytes_for_integral!(i32, I32_SERIALIZED_LENGTH);
impl_to_from_bytes_for_integral!(i64, I64_SERIALIZED_LENGTH);
impl_to_from_bytes_for_integral!(u32, U32_SERIALIZED_LENGTH);
impl_to_from_bytes_for_integral!(u64, U64_SERIALIZED_LENGTH);

impl<T: ToBytes> ToBytes for Vec<T> {
    default fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let serialized_length = self.serialized_length();
        if serialized_length >= u32::max_value() as usize {
            return Err(Error::OutOfMemoryError);
        }

        let mut result = Vec::with_capacity(serialized_length);
        result.append(&mut (self.len() as u32).to_bytes()?);

        for item in self.iter() {
            result.append(&mut item.to_bytes()?);
        }

        Ok(result)
    }

    default fn into_bytes(self) -> Result<Vec<u8>, Error> {
        let serialized_length = self.serialized_length();
        if serialized_length >= u32::max_value() as usize {
            return Err(Error::OutOfMemoryError);
        }

        let mut result = Vec::with_capacity(serialized_length);
        result.append(&mut (self.len() as u32).to_bytes()?);

        for item in self {
            result.append(&mut item.into_bytes()?);
        }

        Ok(result)
    }

    default fn serialized_length(&self) -> usize {
        U32_SERIALIZED_LENGTH + self.iter().map(ToBytes::serialized_length).sum::<usize>()
    }

    default fn uref_offsets(&self) -> Vec<u32> {
        let mut result = vec![];
        let mut running_offset = U32_SERIALIZED_LENGTH as u32;
        for item in self {
            for offset in item.uref_offsets() {
                result.push(running_offset + offset);
            }
            running_offset += item.serialized_length() as u32;
        }
        result
    }
}

impl<T: FromBytes> FromBytes for Vec<T> {
    default fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (size, mut stream) = u32::from_bytes(bytes)?;

        let mut result = Vec::new();
        result.try_reserve_exact(size as usize)?;

        for _ in 0..size {
            let (value, remainder) = T::from_bytes(stream)?;
            result.push(value);
            stream = remainder;
        }

        Ok((result, stream))
    }
}

impl ToBytes for Vec<u8> {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let serialized_length = self.serialized_length();
        if serialized_length >= u32::max_value() as usize {
            return Err(Error::OutOfMemoryError);
        }

        let mut result = Vec::with_capacity(serialized_length);
        result.append(&mut (self.len() as u32).to_bytes()?);
        result.extend(self);
        Ok(result)
    }

    fn into_bytes(mut self) -> Result<Vec<u8>, Error> {
        let serialized_length = self.serialized_length();
        if serialized_length >= u32::max_value() as usize {
            return Err(Error::OutOfMemoryError);
        }

        let mut result = Vec::with_capacity(serialized_length);
        result.append(&mut (self.len() as u32).to_bytes()?);
        result.append(&mut self);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        U32_SERIALIZED_LENGTH + self.len()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        vec![]
    }
}

impl FromBytes for Vec<u8> {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (size, remainder) = u32::from_bytes(bytes)?;

        let mut result = Vec::new();
        result.try_reserve_exact(size as usize)?;
        result.resize_with(size as usize, Default::default);

        let (bytes, remainder) = safe_split_at(remainder, size as usize)?;
        result.copy_from_slice(bytes);
        Ok((result, remainder))
    }

    fn from_vec(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), Error> {
        let (size, mut stream) = u32::from_vec(bytes)?;

        if size as usize > stream.len() {
            Err(Error::EarlyEndOfStream)
        } else {
            let remainder = stream.split_off(size as usize);
            Ok((stream, remainder))
        }
    }
}

impl<T: ToBytes> ToBytes for Option<T> {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        match self {
            Some(v) => {
                let mut value = v.to_bytes()?;
                if value.len() >= u32::max_value() as usize - U8_SERIALIZED_LENGTH {
                    return Err(Error::OutOfMemoryError);
                }
                let mut result: Vec<u8> = Vec::with_capacity(U8_SERIALIZED_LENGTH + value.len());
                result.append(&mut 1u8.to_bytes()?);
                result.append(&mut value);
                Ok(result)
            }
            // In the case of None there is no value to serialize, but we still
            // need to write out a tag to indicate which variant we are using
            None => Ok(0u8.to_bytes()?),
        }
    }

    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
            + match self {
                Some(v) => v.serialized_length(),
                None => 0,
            }
    }

    fn uref_offsets(&self) -> Vec<u32> {
        match self {
            Some(v) => {
                let offsets = v.uref_offsets();
                offsets
                    .into_iter()
                    .map(|offset| offset + U8_SERIALIZED_LENGTH as u32)
                    .collect()
            }
            None => vec![],
        }
    }
}

impl<T: FromBytes> FromBytes for Option<T> {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (tag, rem): (u8, &[u8]) = FromBytes::from_bytes(bytes)?;
        match tag {
            0 => Ok((None, rem)),
            1 => {
                let (t, rem): (T, &[u8]) = FromBytes::from_bytes(rem)?;
                Ok((Some(t), rem))
            }
            _ => Err(Error::FormattingError),
        }
    }
}

macro_rules! impl_to_from_bytes_for_array {
    ($($N:literal)+) => {
        $(
            impl<T: ToBytes> ToBytes for [T; $N] {
                default fn to_bytes(&self) -> Result<Vec<u8>, Error> {
                    // Approximation, as `size_of::<T>()` is only roughly equal to the serialized
                    // size of `T`.
                    let approx_size = self.len() * size_of::<T>();
                    if approx_size >= u32::max_value() as usize {
                        return Err(Error::OutOfMemoryError);
                    }

                    let mut result = Vec::with_capacity(approx_size);
                    result.append(&mut ($N as u32).to_bytes()?);

                    for item in self.iter() {
                        result.append(&mut item.to_bytes()?);
                    }

                    Ok(result)
                }

                default fn serialized_length(&self) -> usize {
                    U32_SERIALIZED_LENGTH +
                        self.iter().map(ToBytes::serialized_length).sum::<usize>()
                }

                default fn uref_offsets(&self) -> Vec<u32> {
                    let mut result = vec![];
                    let mut running_offset = U32_SERIALIZED_LENGTH as u32;
                    for item in self.iter() {
                        for offset in item.uref_offsets() {
                            result.push(running_offset + offset);
                        }
                        running_offset += item.serialized_length() as u32;
                    }
                    result
                }
            }

            impl<T: FromBytes> FromBytes for [T; $N] {
                default fn from_bytes(mut bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
                    let (size, remainder) = u32::from_bytes(bytes)?;
                    bytes = remainder;
                    if size != $N as u32 {
                        return Err(Error::FormattingError);
                    }

                    let mut result: MaybeUninit<[T; $N]> = MaybeUninit::uninit();
                    let result_ptr = result.as_mut_ptr() as *mut T;
                    unsafe {
                        for i in 0..$N {
                            let (t, remainder) = match T::from_bytes(bytes) {
                                Ok(success) => success,
                                Err(error) => {
                                    for j in 0..i {
                                        result_ptr.add(j).drop_in_place();
                                    }
                                    return Err(error);
                                }
                            };
                            result_ptr.add(i).write(t);
                            bytes = remainder;
                        }
                        Ok((result.assume_init(), bytes))
                    }
                }
            }
        )+
    }
}

impl_to_from_bytes_for_array! {
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
    64 128 256 512
}

macro_rules! impl_to_from_bytes_for_byte_array {
    ($($len:expr)+) => {
        $(
            impl ToBytes for [u8; $len] {
                fn to_bytes(&self) -> Result<Vec<u8>, Error> {
                    Ok(self.to_vec())
                }

                fn serialized_length(&self) -> usize { $len }
            }

            impl FromBytes for [u8; $len] {
                fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
                    let (bytes, rem) = safe_split_at(bytes, $len)?;
                    let mut result = [0u8; $len];
                    result.copy_from_slice(bytes);
                    Ok((result, rem))
                }
            }
        )+
    }
}

impl_to_from_bytes_for_byte_array! {
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
    64 128 256 512
}

impl ToBytes for String {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        self.as_str().to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.as_str().serialized_length()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        vec![]
    }
}

impl FromBytes for String {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (str_bytes, rem): (Vec<u8>, &[u8]) = FromBytes::from_bytes(bytes)?;
        let result = String::from_utf8(str_bytes).map_err(|_| Error::FormattingError)?;
        Ok((result, rem))
    }
}

impl ToBytes for () {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        Ok(Vec::new())
    }

    fn serialized_length(&self) -> usize {
        0
    }

    fn uref_offsets(&self) -> Vec<u32> {
        vec![]
    }
}

impl FromBytes for () {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        Ok(((), bytes))
    }
}

impl<K, V> ToBytes for BTreeMap<K, V>
where
    K: ToBytes,
    V: ToBytes,
{
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let num_keys = self.len() as u32;
        let bytes = self
            .iter()
            .map(move |(k, v)| {
                let k_bytes = k.to_bytes().map_err(Error::from);
                let v_bytes = v.to_bytes().map_err(Error::from);
                // For each key and value pair create a vector of
                // serialization results
                let mut vs = Vec::with_capacity(2);
                vs.push(k_bytes);
                vs.push(v_bytes);
                vs
            })
            // Flatten iterable of key and value pairs
            .flatten()
            // Collect into a single Result of bytes (if successful)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten();

        let (lower_bound, _upper_bound) = bytes.size_hint();
        if lower_bound >= u32::max_value() as usize - U32_SERIALIZED_LENGTH {
            return Err(Error::OutOfMemoryError);
        }
        let mut result: Vec<u8> = Vec::with_capacity(U32_SERIALIZED_LENGTH + lower_bound);
        result.append(&mut num_keys.to_bytes()?);
        result.extend(bytes);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        U32_SERIALIZED_LENGTH
            + self
                .iter()
                .map(|(key, value)| key.serialized_length() + value.serialized_length())
                .sum::<usize>()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        let mut result = vec![];
        let mut running_offset = U32_SERIALIZED_LENGTH as u32;

        for (key, value) in self.iter() {
            for offset in key.uref_offsets() {
                result.push(running_offset + offset);
            }
            running_offset += key.serialized_length() as u32;

            for offset in value.uref_offsets() {
                result.push(running_offset + offset);
            }
            running_offset += value.serialized_length() as u32;
        }

        result
    }
}

impl<K, V> FromBytes for BTreeMap<K, V>
where
    K: FromBytes + Ord,
    V: FromBytes,
{
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (num_keys, mut stream): (u32, &[u8]) = FromBytes::from_bytes(bytes)?;
        let mut result = BTreeMap::new();
        for _ in 0..num_keys {
            let (k, rem): (K, &[u8]) = FromBytes::from_bytes(stream)?;
            let (v, rem): (V, &[u8]) = FromBytes::from_bytes(rem)?;
            result.insert(k, v);
            stream = rem;
        }
        Ok((result, stream))
    }
}

impl ToBytes for str {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        if self.len() >= u32::max_value() as usize - U32_SERIALIZED_LENGTH {
            return Err(Error::OutOfMemoryError);
        }
        self.as_bytes().to_vec().into_bytes()
    }

    fn serialized_length(&self) -> usize {
        U32_SERIALIZED_LENGTH + self.as_bytes().len()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        vec![]
    }
}

impl ToBytes for &str {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        (*self).to_bytes()
    }

    fn serialized_length(&self) -> usize {
        (*self).serialized_length()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        vec![]
    }
}

impl<T: ToBytes, E: ToBytes> ToBytes for Result<T, E> {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let (mut variant, mut value) = match self {
            Ok(result) => (1u8.to_bytes()?, result.to_bytes()?),
            Err(error) => (0u8.to_bytes()?, error.to_bytes()?),
        };
        let mut result: Vec<u8> = Vec::with_capacity(U8_SERIALIZED_LENGTH + value.len());
        result.append(&mut variant);
        result.append(&mut value);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
            + match self {
                Ok(ok) => ok.serialized_length(),
                Err(error) => error.serialized_length(),
            }
    }

    fn uref_offsets(&self) -> Vec<u32> {
        let offsets = match self {
            Ok(ok) => ok.uref_offsets(),
            Err(error) => error.uref_offsets(),
        };
        offsets
            .into_iter()
            .map(|offset| offset + U8_SERIALIZED_LENGTH as u32)
            .collect()
    }
}

impl<T: FromBytes, E: FromBytes> FromBytes for Result<T, E> {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (variant, rem): (u8, &[u8]) = FromBytes::from_bytes(bytes)?;
        match variant {
            0 => {
                let (value, rem): (E, &[u8]) = FromBytes::from_bytes(rem)?;
                Ok((Err(value), rem))
            }
            1 => {
                let (value, rem): (T, &[u8]) = FromBytes::from_bytes(rem)?;
                Ok((Ok(value), rem))
            }
            _ => Err(Error::FormattingError),
        }
    }
}

impl ToBytes for SemVer {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut ret: Vec<u8> = Vec::with_capacity(SEM_VER_SERIALIZED_LENGTH);
        ret.append(&mut self.major.to_bytes()?);
        ret.append(&mut self.minor.to_bytes()?);
        ret.append(&mut self.patch.to_bytes()?);
        Ok(ret)
    }

    fn serialized_length(&self) -> usize {
        SEM_VER_SERIALIZED_LENGTH
    }

    fn uref_offsets(&self) -> Vec<u32> {
        vec![]
    }
}

impl FromBytes for SemVer {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (major, rem): (u32, &[u8]) = FromBytes::from_bytes(bytes)?;
        let (minor, rem): (u32, &[u8]) = FromBytes::from_bytes(rem)?;
        let (patch, rem): (u32, &[u8]) = FromBytes::from_bytes(rem)?;
        Ok((SemVer::new(major, minor, patch), rem))
    }
}

impl ToBytes for ProtocolVersion {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        self.value().to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.value().serialized_length()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        vec![]
    }
}

impl FromBytes for ProtocolVersion {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (version, rem): (SemVer, &[u8]) = FromBytes::from_bytes(bytes)?;
        let protocol_version = ProtocolVersion::new(version);
        Ok((protocol_version, rem))
    }
}

impl<T1: ToBytes> ToBytes for (T1,) {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        self.0.to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        self.0.uref_offsets()
    }
}

impl<T1: FromBytes> FromBytes for (T1,) {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        Ok(((t1,), remainder))
    }
}

impl<T1: ToBytes, T2: ToBytes> ToBytes for (T1, T2) {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut result = Vec::new();
        result.append(&mut self.0.to_bytes()?);
        result.append(&mut self.1.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length() + self.1.serialized_length()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        self.0
            .uref_offsets()
            .into_iter()
            .chain(
                self.1
                    .uref_offsets()
                    .into_iter()
                    .map(|offset| offset + self.0.serialized_length() as u32),
            )
            .collect()
    }
}

impl<T1: FromBytes, T2: FromBytes> FromBytes for (T1, T2) {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        let (t2, remainder) = T2::from_bytes(remainder)?;
        Ok(((t1, t2), remainder))
    }
}

impl<T1: ToBytes, T2: ToBytes, T3: ToBytes> ToBytes for (T1, T2, T3) {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut result = Vec::new();
        result.append(&mut self.0.to_bytes()?);
        result.append(&mut self.1.to_bytes()?);
        result.append(&mut self.2.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length() + self.1.serialized_length() + self.2.serialized_length()
    }

    fn uref_offsets(&self) -> Vec<u32> {
        self.0
            .uref_offsets()
            .into_iter()
            .chain(
                self.1
                    .uref_offsets()
                    .into_iter()
                    .map(|offset| offset + self.0.serialized_length() as u32),
            )
            .chain(self.2.uref_offsets().into_iter().map(|offset| {
                offset + self.0.serialized_length() as u32 + self.1.serialized_length() as u32
            }))
            .collect()
    }
}

impl<T1: FromBytes, T2: FromBytes, T3: FromBytes> FromBytes for (T1, T2, T3) {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (t1, remainder) = T1::from_bytes(bytes)?;
        let (t2, remainder) = T2::from_bytes(remainder)?;
        let (t3, remainder) = T3::from_bytes(remainder)?;
        Ok(((t1, t2, t3), remainder))
    }
}

#[doc(hidden)]
/// Returns `true` if a we can serialize and then deserialize a value
pub fn test_serialization_roundtrip<T>(t: &T)
where
    T: ToBytes + FromBytes + PartialEq,
{
    let serialized = ToBytes::to_bytes(t).expect("Unable to serialize data");
    assert_eq!(serialized.len(), t.serialized_length(), "{:?}", serialized);
    let deserialized = deserialize::<T>(serialized).expect("Unable to deserialize data");
    assert!(*t == deserialized)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    #[test]
    fn check_array_from_bytes_doesnt_leak() {
        thread_local!(static INSTANCE_COUNT: RefCell<usize> = RefCell::new(0));
        const MAX_INSTANCES: usize = 10;

        struct LeakChecker;

        impl FromBytes for LeakChecker {
            fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
                let instance_num = INSTANCE_COUNT.with(|count| *count.borrow());
                if instance_num >= MAX_INSTANCES {
                    Err(Error::FormattingError)
                } else {
                    INSTANCE_COUNT.with(|count| *count.borrow_mut() += 1);
                    Ok((LeakChecker, bytes))
                }
            }
        }

        impl Drop for LeakChecker {
            fn drop(&mut self) {
                INSTANCE_COUNT.with(|count| *count.borrow_mut() -= 1);
            }
        }

        // Check we can construct an array of `MAX_INSTANCES` of `LeakChecker`s.
        {
            let bytes = (MAX_INSTANCES as u32).to_bytes().unwrap();
            let _array = <[LeakChecker; MAX_INSTANCES]>::from_bytes(&bytes).unwrap();
            // Assert `INSTANCE_COUNT == MAX_INSTANCES`
            INSTANCE_COUNT.with(|count| assert_eq!(MAX_INSTANCES, *count.borrow()));
        }

        // Assert the `INSTANCE_COUNT` has dropped to zero again.
        INSTANCE_COUNT.with(|count| assert_eq!(0, *count.borrow()));

        // Try to construct an array of `LeakChecker`s where the `MAX_INSTANCES + 1`th instance
        // returns an error.
        let bytes = (MAX_INSTANCES as u32 + 1).to_bytes().unwrap();
        let result = <[LeakChecker; MAX_INSTANCES + 1]>::from_bytes(&bytes);
        assert!(result.is_err());

        // Assert the `INSTANCE_COUNT` has dropped to zero again.
        INSTANCE_COUNT.with(|count| assert_eq!(0, *count.borrow()));
    }
}

#[allow(clippy::unnecessary_operation)]
#[cfg(test)]
mod proptests {
    use proptest::{collection::vec, prelude::*};

    use crate::{bytesrepr, gens::*};

    proptest! {
        #[test]
        fn test_bool(u in any::<bool>()) {
            bytesrepr::test_serialization_roundtrip(&u)
        }

        #[test]
        fn test_u8(u in any::<u8>()) {
            bytesrepr::test_serialization_roundtrip(&u)
        }

        #[test]
        fn test_u32(u in any::<u32>()) {
            bytesrepr::test_serialization_roundtrip(&u)
        }

        #[test]
        fn test_i32(u in any::<i32>()) {
            bytesrepr::test_serialization_roundtrip(&u)
        }

        #[test]
        fn test_u64(u in any::<u64>()) {
            bytesrepr::test_serialization_roundtrip(&u)
        }

        #[test]
        fn test_i64(u in any::<i64>()) {
            bytesrepr::test_serialization_roundtrip(&u)
        }

        #[test]
        fn test_u8_slice_32(s in u8_slice_32()) {
            bytesrepr::test_serialization_roundtrip(&s)
        }

        #[test]
        fn test_vec_u8(u in vec(any::<u8>(), 1..100)) {
            bytesrepr::test_serialization_roundtrip(&u)
        }

        #[test]
        fn test_vec_i32(u in vec(any::<i32>(), 1..100)) {
            bytesrepr::test_serialization_roundtrip(&u)
        }

        #[test]
        fn test_vec_vec_u8(u in vec(vec(any::<u8>(), 1..100), 10)) {
            bytesrepr::test_serialization_roundtrip(&u)
        }

        #[test]
        fn test_uref_map(m in named_keys_arb(20)) {
            bytesrepr::test_serialization_roundtrip(&m)
        }

        #[test]
        fn test_array_u8_32(arr in any::<[u8; 32]>()) {
            bytesrepr::test_serialization_roundtrip(&arr)
        }

        #[test]
        fn test_string(s in "\\PC*") {
            bytesrepr::test_serialization_roundtrip(&s)
        }

        #[test]
        fn test_option(o in proptest::option::of(key_arb())) {
            bytesrepr::test_serialization_roundtrip(&o)
        }

        #[test]
        fn test_unit(unit in Just(())) {
            bytesrepr::test_serialization_roundtrip(&unit)
        }

        #[test]
        fn test_u128_serialization(u in u128_arb()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u256_serialization(u in u256_arb()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_u512_serialization(u in u512_arb()) {
            bytesrepr::test_serialization_roundtrip(&u);
        }

        #[test]
        fn test_key_serialization(key in key_arb()) {
            bytesrepr::test_serialization_roundtrip(&key);
        }

        #[test]
        fn test_cl_value_serialization(cl_value in cl_value_arb()) {
            bytesrepr::test_serialization_roundtrip(&cl_value);
        }

        #[test]
        fn test_access_rights(access_right in access_rights_arb()) {
            bytesrepr::test_serialization_roundtrip(&access_right)
        }

        #[test]
        fn test_uref(uref in uref_arb()) {
            bytesrepr::test_serialization_roundtrip(&uref);
        }

        #[test]
        fn test_public_key(pk in public_key_arb()) {
            bytesrepr::test_serialization_roundtrip(&pk)
        }

        #[test]
        fn test_result(result in result_arb()) {
            bytesrepr::test_serialization_roundtrip(&result)
        }

        #[test]
        fn test_phase_serialization(phase in phase_arb()) {
            bytesrepr::test_serialization_roundtrip(&phase)
        }

        #[test]
        fn test_protocol_version(protocol_version in protocol_version_arb()) {
            bytesrepr::test_serialization_roundtrip(&protocol_version)
        }

        #[test]
        fn test_sem_ver(sem_ver in sem_ver_arb()) {
            bytesrepr::test_serialization_roundtrip(&sem_ver)
        }

        #[test]
        fn test_tuple1(t in (any::<u8>(),)) {
            bytesrepr::test_serialization_roundtrip(&t)
        }

        #[test]
        fn test_tuple2(t in (any::<u8>(),any::<u32>())) {
            bytesrepr::test_serialization_roundtrip(&t)
        }

        #[test]
        fn test_tuple3(t in (any::<u8>(),any::<u32>(),any::<i32>())) {
            bytesrepr::test_serialization_roundtrip(&t)
        }
    }
}
