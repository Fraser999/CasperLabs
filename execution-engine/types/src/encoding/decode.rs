use alloc::str;
use core::{mem, u32};

use serde::de::{
    Deserialize, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};

use super::{Error, Result};

pub(super) struct Deserializer<'de> {
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    /// Creates a new Deserializer that will read from the given slice.
    #[inline]
    pub(super) fn new(input: &'de [u8]) -> Result<Self> {
        if input.len() > u32::max_value() as usize {
            return Err(Error::SizeLimit);
        }

        Ok(Deserializer { input })
    }

    pub(super) fn input_slice_is_empty(self) -> Result<()> {
        if self.input.is_empty() {
            Ok(())
        } else {
            Err(Error::LeftOverBytes(self.input.len()))
        }
    }

    #[inline]
    /// Splits off the first `count` bytes from `self.input`.
    fn take_bytes(&mut self, count: u32) -> Result<&'de [u8]> {
        if count as usize > self.input.len() {
            Err(Error::EndOfSlice)
        } else {
            let (removed, remainder) = self.input.split_at(count as usize);
            self.input = remainder;
            Ok(removed)
        }
    }

    /// Removes the first byte from `self.input` and returns it.
    #[inline]
    fn deserialize_byte(&mut self) -> Result<u8> {
        match self.input.split_first() {
            None => Err(Error::EndOfSlice),
            Some((byte, remainder)) => {
                self.input = remainder;
                Ok(*byte)
            }
        }
    }

    #[inline]
    /// Removes the first 4 bytes from `self.input` and returns them parsed into a `u32`.
    fn deserialize_length(&mut self) -> Result<u32> {
        const LENGTH: usize = mem::size_of::<u32>();
        let mut result = [0; LENGTH];
        let bytes = self.take_bytes(LENGTH as u32)?;
        result.copy_from_slice(bytes);
        Ok(<u32>::from_le_bytes(result))
    }
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.deserialize_byte()? {
            1 => visitor.visit_bool(true),
            0 => visitor.visit_bool(false),
            value => Err(Error::InvalidBoolEncoding(value)),
        }
    }

    #[inline]
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(self.deserialize_byte()?)
    }

    #[inline]
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        const LENGTH: usize = mem::size_of::<u16>();
        let mut result = [0; LENGTH];
        let bytes = self.take_bytes(LENGTH as u32)?;
        result.copy_from_slice(bytes);
        visitor.visit_u16(<u16>::from_le_bytes(result))
    }

    #[inline]
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(self.deserialize_length()?)
    }

    #[inline]
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        const LENGTH: usize = mem::size_of::<u64>();
        let mut result = [0; LENGTH];
        let bytes = self.take_bytes(LENGTH as u32)?;
        result.copy_from_slice(bytes);
        visitor.visit_u64(<u64>::from_le_bytes(result))
    }

    #[inline]
    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(self.deserialize_byte()? as i8)
    }

    #[inline]
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        const LENGTH: usize = mem::size_of::<i16>();
        let mut result = [0; LENGTH];
        let bytes = self.take_bytes(LENGTH as u32)?;
        result.copy_from_slice(bytes);
        visitor.visit_i16(<i16>::from_le_bytes(result))
    }

    #[inline]
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        const LENGTH: usize = mem::size_of::<i32>();
        let mut result = [0; LENGTH];
        let bytes = self.take_bytes(LENGTH as u32)?;
        result.copy_from_slice(bytes);
        visitor.visit_i32(<i32>::from_le_bytes(result))
    }

    #[inline]
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        const LENGTH: usize = mem::size_of::<i64>();
        let mut result = [0; LENGTH];
        let bytes = self.take_bytes(LENGTH as u32)?;
        result.copy_from_slice(bytes);
        visitor.visit_i64(<i64>::from_le_bytes(result))
    }

    fn deserialize_f32<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::Unsupported)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::Unsupported)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let length = self.deserialize_length()?;
        let bytes = self.take_bytes(length)?;
        let string = str::from_utf8(bytes)?;
        visitor.visit_borrowed_str(string)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let first_byte = self.deserialize_byte()?;
        let width = utf8_char_width(first_byte);

        if width == 1 {
            return visitor.visit_char(first_byte as char);
        }
        if width == 0 {
            return Err(Error::InvalidCharEncoding);
        }

        let mut buffer = [first_byte; 4];
        let remaining_len = width - 1;
        let remaining_bytes = self.take_bytes(remaining_len as u32)?;
        buffer[1..remaining_len].copy_from_slice(remaining_bytes);

        let res = str::from_utf8(&buffer[..width])
            .ok()
            .and_then(|string| string.chars().next())
            .ok_or_else(|| Error::InvalidCharEncoding)?;
        visitor.visit_char(res)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let length = self.deserialize_length()?;
        let bytes = self.take_bytes(length)?;
        visitor.visit_borrowed_bytes(bytes)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let length = self.deserialize_length()?;
        let bytes = self.take_bytes(length)?;
        visitor.visit_bytes(bytes)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = u8::deserialize(&mut *self)?;
        match value {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(&mut *self),
            _ => Err(Error::InvalidTagEncoding(value)),
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let length = self.deserialize_length()?;
        self.deserialize_tuple(length as usize, visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _enum: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_enum(self)
    }

    #[inline]
    fn deserialize_tuple<V: Visitor<'de>>(self, length: usize, visitor: V) -> Result<V::Value> {
        visitor.visit_seq(Access {
            deserializer: self,
            length,
        })
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        length: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_tuple(length, visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let length = self.deserialize_length()? as usize;
        visitor.visit_map(Access {
            deserializer: self,
            length,
        })
    }

    fn deserialize_struct<V>(
        self,
        _name: &str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::Unsupported)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::Unsupported)
    }

    #[inline]
    fn deserialize_ignored_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::Unsupported)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'de, 'a> EnumAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<(T::Value, Self::Variant)> {
        let index: u8 = self.deserialize_byte()?;
        let value: Result<_> = seed.deserialize((index as u32).into_deserializer());
        Ok((value?, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        DeserializeSeed::deserialize(seed, self)
    }

    fn tuple_variant<V: Visitor<'de>>(self, length: usize, visitor: V) -> Result<V::Value> {
        serde::Deserializer::deserialize_tuple(self, length, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        serde::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}

struct Access<'de, 'a> {
    deserializer: &'a mut Deserializer<'de>,
    length: usize,
}

impl<'de, 'a> SeqAccess<'de> for Access<'de, 'a> {
    type Error = Error;

    // Inlining here is crucial because this is called from our crate through serde, thus needs
    // cross-crate inlining.
    #[inline]
    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.length > 0 {
            self.length -= 1;
            let value = DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.length)
    }
}

impl<'de, 'a> MapAccess<'de> for Access<'de, 'a> {
    type Error = Error;

    fn next_key_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.length > 0 {
            self.length -= 1;
            let key = DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<T::Value> {
        let value = DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
        Ok(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.length)
    }
}

// Copied from https://doc.rust-lang.org/src/core/str/mod.rs.html#1685-1710
static UTF8_CHAR_WIDTH: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline]
fn utf8_char_width(byte: u8) -> usize {
    UTF8_CHAR_WIDTH[byte as usize] as usize
}
