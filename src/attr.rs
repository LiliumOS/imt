use core::any::Any;
use std::{borrow::Cow, marker::PhantomData};

use crate::uuid::Uuid;
use bincode::{
    BorrowDecode, Decode, Encode,
    de::read::Reader,
    enc::{Encoder, write::Writer},
    error::{DecodeError, EncodeError},
};

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    pub struct AttributeFlags : u32 {
        const IGNORE = 0x0000_0001;

        const TYPE_MASK = 0xFF80_0000;
    }
}

impl<C> Decode<C> for AttributeFlags {
    fn decode<D: bincode::de::Decoder<Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let v = u32::decode(decoder)?;

        AttributeFlags::from_bits(v).ok_or_else(|| {
            DecodeError::OtherString(format!(
                "Flags {:?} sets illegal flags",
                AttributeFlags::from_bits_retain(v)
            ))
        })
    }
}

impl<'de, C> BorrowDecode<'de, C> for AttributeFlags {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Self::decode(decoder)
    }
}

impl Encode for AttributeFlags {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        self.bits().encode(encoder)
    }
}

#[derive(Clone, Encode)]
pub struct Attribute<Targ> {
    id: Uuid,
    flags: AttributeFlags,
    payload: ErasedAttributeContent<Targ>,
}

impl<Targ> core::fmt::Debug for Attribute<Targ> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.payload {
            ErasedAttributeContent::Real(attr, _) => f
                .debug_struct("Attribute")
                .field("flags", &self.flags)
                .field("payload", &attr)
                .finish_non_exhaustive(),
            ErasedAttributeContent::Unknown(_) => f
                .debug_struct("Attribute")
                .field("flags", &self.flags)
                .field(
                    "payload",
                    &format_args!("Unknown atribute {:#?}", self.id.0),
                )
                .finish_non_exhaustive(),
        }
    }
}

impl<Targ: AttributeTarget> Attribute<Targ> {
    pub fn new<T: Target<Targ>>(x: T) -> Self {
        Attribute {
            id: T::ID,
            flags: AttributeFlags::empty(),
            payload: ErasedAttributeContent::Real(Box::new(x), PhantomData),
        }
    }

    pub fn downcast<T: Target<Targ>>(&self) -> Option<&T> {
        if self.id != T::ID {
            return None;
        }

        match &self.payload {
            ErasedAttributeContent::Real(real, _) => <dyn Any>::downcast_ref(real),
            _ => None,
        }
    }

    pub const fn id(&self) -> &Uuid {
        &self.id
    }

    pub const fn flags(&self) -> &AttributeFlags {
        &self.flags
    }

    pub const fn flags_mut(&mut self) -> &mut AttributeFlags {
        &mut self.flags
    }

    pub const fn with_flags(mut self, additional_flags: AttributeFlags) -> Self {
        self.flags = self.flags.union(additional_flags);
        self
    }
}

impl<C, Targ: AttributeTarget> Decode<C> for Attribute<Targ> {
    fn decode<D: bincode::de::Decoder<Context = C>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let id = Uuid::decode(decoder)?;
        let flags = AttributeFlags::decode(decoder)?;

        let mut decoder = decoder.with_context((flags, id));

        let payload = ErasedAttributeContent::decode(&mut decoder)?;

        Ok(Self { id, flags, payload })
    }
}

impl<'de, C, Targ: AttributeTarget> BorrowDecode<'de, C> for Attribute<Targ> {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Self::decode(decoder)
    }
}

enum ErasedAttributeContent<Targ> {
    Real(Box<dyn DynAttr>, PhantomData<Targ>),
    Unknown(Vec<u8>),
}

impl<Targ> Clone for ErasedAttributeContent<Targ> {
    fn clone(&self) -> Self {
        match self {
            Self::Real(attr, phantom) => Self::Real(attr.clone_box(), *phantom),
            Self::Unknown(bytes) => Self::Unknown(bytes.clone()),
        }
    }
}

impl<Targ> Encode for ErasedAttributeContent<Targ> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let bytes: Cow<[u8]> = match self {
            Self::Real(attr, _) => Cow::Owned(attr.to_bytes()?),
            Self::Unknown(bytes) => Cow::Borrowed(&**bytes),
        };

        let len: u32 = bytes.len().try_into().map_err(|_| {
            EncodeError::Other("Attribute length limit supports no more than 2^32 bytes")
        })?;

        len.encode(encoder)?;
        encoder.writer().write(&bytes)
    }
}

impl<Targ: AttributeTarget> Decode<(AttributeFlags, Uuid)> for ErasedAttributeContent<Targ> {
    fn decode<D: bincode::de::Decoder<Context = (AttributeFlags, Uuid)>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let (flags, id) = *decoder.context();

        let mut attr = create_attribute_blob::<Targ>(id);

        let data_len = u32::decode(decoder)? as usize;

        let mut data = Vec::with_capacity(data_len);
        data.resize(data_len, 0u8);
        decoder.reader().read(&mut data)?;

        match attr {
            Some(mut attr) => {
                attr.from_bytes(&data)?;

                Ok(Self::Real(attr, PhantomData))
            }
            None => {
                if !flags.contains(AttributeFlags::IGNORE) {
                    return Err(DecodeError::OtherString(format!(
                        "Non-ignorable attribute with id {id} is not recognized"
                    )));
                }

                Ok(Self::Unknown(data))
            }
        }
    }
}

pub trait AttributeTarget {
    const KIND: AttributeTargetKind;
}

macro_rules! def_attribute_targets {
    ($(target $name:ident;)*) => {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        pub enum AttributeTargetKind {
            $($name),*
        }

        $(impl AttributeTarget for $name {
            const KIND: AttributeTargetKind = AttributeTargetKind :: $name;
        })*
    };
}

use crate::{
    file::{File, UseItem},
    tydef::{Enum, Field, Struct, TypeAlias, Union, Variant},
    uses::Param,
    value::{Const, Function},
};

def_attribute_targets! {
    target File;
    target UseItem;
    target Struct;
    target Union;
    target Enum;
    target TypeAlias;
    target Field;
    target Variant;
    target Const;
    target Function;
    target Param;
}

pub trait Target<T: AttributeTarget>: AttributeType {}

pub trait AttributeType: Any + Clone + Encode + Decode<()> + Default + core::fmt::Debug {
    const ID: Uuid;
    const TARGET: Option<&[AttributeTargetKind]>;
}

trait DynAttr: Any {
    fn clone_box(&self) -> Box<dyn DynAttr>;
    fn from_bytes(&mut self, bytes: &[u8]) -> Result<(), DecodeError>;
    fn to_bytes(&self) -> Result<Vec<u8>, EncodeError>;
    fn fmt_debug<'a>(&self, f: &mut core::fmt::Formatter<'a>) -> core::fmt::Result;
}

impl<A: AttributeType> DynAttr for A {
    fn clone_box(&self) -> Box<dyn DynAttr> {
        Box::new(self.clone())
    }
    fn from_bytes(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        let (val, read) = bincode::decode_from_slice(bytes, crate::config::format_config())?;
        if read != bytes.len() {
            return Err(DecodeError::OtherString(format!(
                "Extra slop found on stream. Expected {} bytes, got {read}",
                bytes.len()
            )));
        }
        *self = val;
        Ok(())
    }

    fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::encode_to_vec(self, crate::config::format_config())
    }

    fn fmt_debug<'a>(&self, f: &mut core::fmt::Formatter<'a>) -> core::fmt::Result {
        self.fmt(f)
    }
}

impl core::fmt::Debug for dyn DynAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_debug(f)
    }
}

macro_rules! impl_target {
    (attr $ty:path [$($target:ident),*]) => {
        $(impl Target<$target> for $ty{})*
    };
    (attr $ty:path) => {
        impl<T: AttributeTarget> Target<T> for $ty{}
    }
}

macro_rules! attribute_types {
    ($(attr $ty:path = $id:literal $([$($target:ident),* $(,)?])?;)*) => {
        $(
            impl AttributeType for $ty {
                const ID: Uuid = Uuid::parse($id);
                const TARGET: Option<&[AttributeTargetKind]> = ($(Some(&[$(AttributeTargetKind:: $target),*] as &[AttributeTargetKind]),)? None::<&[AttributeTargetKind]>,).0;
            }
            impl_target!(attr $ty $([$($target),*])?);
        )*

        fn create_attribute_blob<__T: AttributeTarget>(id: Uuid) -> Option<Box<dyn DynAttr>> {
            match id {
                $(<$ty as AttributeType>::ID if (
                    match <$ty as AttributeType>::TARGET {
                        Some(arr) => {
                            arr.contains(&<__T as AttributeTarget>::KIND)
                        }
                        None => true
                    }
                ) => Some(Box::new(<$ty as Default>::default())),)*
                _ => None,
            }
        }
    };
}

// v5 based on fcdc6c4f-f218-5a30-a2e5-7e8d7d2a38a6
attribute_types! {
    attr types::SafetyHint = "8649000c-291a-566c-b171-0da33515ea61" [Function];
    attr types::OptionType = "74404322-8d86-5623-93b0-2a8659f9cd09" [Struct];
    attr types::PolymorphicOption = "3072a4e5-598e-55dc-9824-bbc1da1ccaea" [Struct];
    attr types::ItemDoc = "a5a3cce8-4f49-5084-9761-36603109808a";
    attr types::SubsystemDescriptor = "50f98361-bbf6-5f10-8594-7354b4c7c313" [File];
    attr types::SystemFunction = "c130fb9b-ed3f-55e7-9bf9-2ae163bfc4d3" [Function];
    attr types::ExportInline = "df372d18-045d-5d4e-8aad-26db0300c707" [UseItem];
    attr types::DefinesBuiltinTypes = "360cb09a-155e-5bc9-ac7b-d8cb6662687a" [File];
    attr types::ToolComment = "d6ade778-923c-573d-8c88-948fb053d49b" [File];
    attr types::Align = "c9c12154-f381-5d48-88e1-ce31d9d1bd1f" [Struct, Union];
    attr types::Synthetic = "5d4ceb6f-dc75-581c-ba8e-d014a77091fe";
}

pub mod types;
