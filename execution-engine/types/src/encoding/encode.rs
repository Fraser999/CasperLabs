use alloc::vec::Vec;
use core::{fmt::Display, mem, u32};

use super::{Error, Result};

const MAX_INDEX: u32 = 255;

pub(super) struct Serializer {
    output: Vec<u8>,
}

impl Serializer {
    pub(super) fn new(required_length: u32) -> Self {
        let output = Vec::with_capacity(required_length as usize);
        Serializer { output }
    }

    pub(super) fn take_output(self) -> Vec<u8> {
        self.output
    }

    fn serialize_length(&mut self, length: usize) {
        self.output
            .extend_from_slice((length as u32).to_le_bytes().as_ref());
    }

    fn serialize_index(&mut self, index: u32) -> Result<()> {
        if index > MAX_INDEX {
            return Err(Error::ExcessiveDiscriminants);
        }
        self.output.push(index as u8);
        Ok(())
    }
}

impl<'a> serde::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_bool(self, value: bool) -> Result<()> {
        self.output.push(u8::from(value));
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> Result<()> {
        self.output.push(value);
        Ok(())
    }

    fn serialize_u16(self, value: u16) -> Result<()> {
        self.output.extend_from_slice(value.to_le_bytes().as_ref());
        Ok(())
    }

    fn serialize_u32(self, value: u32) -> Result<()> {
        self.output.extend_from_slice(value.to_le_bytes().as_ref());
        Ok(())
    }

    fn serialize_u64(self, value: u64) -> Result<()> {
        self.output.extend_from_slice(value.to_le_bytes().as_ref());
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<()> {
        self.output.extend_from_slice(value.to_le_bytes().as_ref());
        Ok(())
    }

    fn serialize_i16(self, value: i16) -> Result<()> {
        self.output.extend_from_slice(value.to_le_bytes().as_ref());
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> Result<()> {
        self.output.extend_from_slice(value.to_le_bytes().as_ref());
        Ok(())
    }

    fn serialize_i64(self, value: i64) -> Result<()> {
        self.output.extend_from_slice(value.to_le_bytes().as_ref());
        Ok(())
    }

    fn serialize_f32(self, _value: f32) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn serialize_f64(self, _value: f64) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn serialize_str(self, value: &str) -> Result<()> {
        self.serialize_length(value.len());
        self.output.extend_from_slice(value.as_bytes());
        Ok(())
    }

    fn serialize_char(self, value: char) -> Result<()> {
        let mut buffer = [0; 4];
        let encoded = value.encode_utf8(&mut buffer).as_bytes();
        self.output.extend_from_slice(encoded);
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        self.serialize_length(value.len());
        self.output.extend_from_slice(value);
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.output.push(0);
        Ok(())
    }

    fn serialize_some<T: serde::Serialize + ?Sized>(self, value: &T) -> Result<()> {
        self.output.push(1);
        value.serialize(self)
    }

    fn serialize_seq(self, length: Option<usize>) -> Result<Self::SerializeSeq> {
        let length = length.ok_or(Error::SequenceMustHaveLength)?;
        self.serialize_length(length);
        Ok(Compound { serializer: self })
    }

    fn serialize_tuple(self, _length: usize) -> Result<Self::SerializeTuple> {
        Ok(Compound { serializer: self })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(Compound { serializer: self })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_index(variant_index)?;
        Ok(Compound { serializer: self })
    }

    fn serialize_map(self, length: Option<usize>) -> Result<Self::SerializeMap> {
        let length = length.ok_or(Error::SequenceMustHaveLength)?;
        self.serialize_length(length);
        Ok(Compound { serializer: self })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStruct> {
        Ok(Compound { serializer: self })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_index(variant_index)?;
        Ok(Compound { serializer: self })
    }

    fn serialize_newtype_struct<T: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()> {
        self.serialize_index(variant_index)?;
        value.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.serialize_index(variant_index)
    }

    fn collect_str<T: Display + ?Sized>(self, _value: &T) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

#[derive(Default)]
pub(super) struct SizeChecker {
    limit: u32,
    total: u32,
}

impl SizeChecker {
    pub(super) fn new() -> Self {
        SizeChecker {
            limit: u32::max_value(),
            total: 0,
        }
    }

    pub(super) fn take_total(self) -> u32 {
        self.total
    }

    fn add_raw(&mut self, size: u64) -> Result<()> {
        if self.limit as u64 >= size {
            self.limit -= size as u32;
        } else {
            return Err(Error::SizeLimit);
        }

        self.total += size as u32;

        Ok(())
    }

    fn add_discriminant(&mut self, _index: u32) -> Result<()> {
        let size = mem::size_of::<u8>() as u64;
        self.add_raw(size)
    }

    fn add_length(&mut self, _length: usize) -> Result<()> {
        let size = mem::size_of::<u32>() as u64;
        self.add_raw(size)
    }
}

impl<'a> serde::Serializer for &'a mut SizeChecker {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SizeCompound<'a>;
    type SerializeTuple = SizeCompound<'a>;
    type SerializeTupleStruct = SizeCompound<'a>;
    type SerializeTupleVariant = SizeCompound<'a>;
    type SerializeMap = SizeCompound<'a>;
    type SerializeStruct = SizeCompound<'a>;
    type SerializeStructVariant = SizeCompound<'a>;

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_bool(self, _: bool) -> Result<()> {
        self.add_raw(mem::size_of::<u8>() as u64)
    }

    fn serialize_u8(self, _: u8) -> Result<()> {
        self.add_raw(mem::size_of::<u8>() as u64)
    }

    fn serialize_u16(self, _: u16) -> Result<()> {
        self.add_raw(mem::size_of::<u16>() as u64)
    }

    fn serialize_u32(self, _: u32) -> Result<()> {
        self.add_raw(mem::size_of::<u32>() as u64)
    }

    fn serialize_u64(self, _: u64) -> Result<()> {
        self.add_raw(mem::size_of::<u64>() as u64)
    }

    fn serialize_i8(self, _: i8) -> Result<()> {
        self.add_raw(mem::size_of::<i8>() as u64)
    }

    fn serialize_i16(self, _: i16) -> Result<()> {
        self.add_raw(mem::size_of::<i16>() as u64)
    }

    fn serialize_i32(self, _: i32) -> Result<()> {
        self.add_raw(mem::size_of::<i32>() as u64)
    }

    fn serialize_i64(self, _: i64) -> Result<()> {
        self.add_raw(mem::size_of::<i64>() as u64)
    }

    fn serialize_f32(self, _: f32) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn serialize_f64(self, _: f64) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn serialize_str(self, value: &str) -> Result<()> {
        self.add_length(value.len())?;
        self.add_raw(value.len() as u64)
    }

    fn serialize_char(self, value: char) -> Result<()> {
        let mut buffer = [0; 4];
        let encoded = value.encode_utf8(&mut buffer).as_bytes();
        self.add_raw(encoded.len() as u64)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        self.add_length(value.len())?;
        self.add_raw(value.len() as u64)
    }

    fn serialize_none(self) -> Result<()> {
        self.add_raw(1)
    }

    fn serialize_some<T: serde::Serialize + ?Sized>(self, value: &T) -> Result<()> {
        self.add_raw(1)?;
        value.serialize(self)
    }

    fn serialize_seq(self, length: Option<usize>) -> Result<Self::SerializeSeq> {
        let length = length.ok_or(Error::SequenceMustHaveLength)?;
        self.add_length(length)?;
        Ok(SizeCompound { serializer: self })
    }

    fn serialize_tuple(self, _length: usize) -> Result<Self::SerializeTuple> {
        Ok(SizeCompound { serializer: self })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(SizeCompound { serializer: self })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.add_discriminant(variant_index)?;
        Ok(SizeCompound { serializer: self })
    }

    fn serialize_map(self, length: Option<usize>) -> Result<Self::SerializeMap> {
        let length = length.ok_or(Error::SequenceMustHaveLength)?;
        self.add_length(length)?;
        Ok(SizeCompound { serializer: self })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStruct> {
        Ok(SizeCompound { serializer: self })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.add_discriminant(variant_index)?;
        Ok(SizeCompound { serializer: self })
    }

    fn serialize_newtype_struct<V: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &V,
    ) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.add_discriminant(variant_index)
    }

    fn serialize_newtype_variant<V: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &V,
    ) -> Result<()> {
        self.add_discriminant(variant_index)?;
        value.serialize(self)
    }

    fn collect_str<T: Display + ?Sized>(self, _value: &T) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

pub struct Compound<'a> {
    serializer: &'a mut Serializer,
}

impl<'a> serde::ser::SerializeSeq for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTuple for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleStruct for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleVariant for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeMap for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<K: ?Sized>(&mut self, value: &K) -> Result<()>
    where
        K: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn serialize_value<V: ?Sized>(&mut self, value: &V) -> Result<()>
    where
        V: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStruct for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStructVariant for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

pub(crate) struct SizeCompound<'a> {
    serializer: &'a mut SizeChecker,
}

impl<'a> serde::ser::SerializeSeq for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTuple for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleStruct for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleVariant for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeMap for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<K: ?Sized>(&mut self, value: &K) -> Result<()>
    where
        K: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn serialize_value<V: ?Sized>(&mut self, value: &V) -> Result<()>
    where
        V: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStruct for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStructVariant for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}
