use std::ops::{Deref, DerefMut};

use bincode::{BorrowDecode, Decode, Encode};

use lilium_sys::uuid::Uuid as Underlying;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Uuid(pub Underlying);

impl Default for Uuid {
    fn default() -> Self {
        Self(Underlying::NIL)
    }
}

impl Uuid {
    pub const fn parse(x: &str) -> Self {
        Self(lilium_sys::uuid::parse_uuid(x))
    }

    pub const fn into_inner(self) -> Underlying {
        self.0
    }

    pub const fn inner(&self) -> &Underlying {
        &self.0
    }

    pub const fn inner_mut(&mut self) -> &mut Underlying {
        &mut self.0
    }
}

impl Deref for Uuid {
    type Target = Underlying;
    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl DerefMut for Uuid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

impl core::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<C> Decode<C> for Uuid {
    fn decode<D: bincode::de::Decoder<Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let lo = u64::decode(decoder)?;
        let hi = u64::decode(decoder)?;

        Ok(Self(Underlying {
            minor: lo,
            major: hi,
        }))
    }
}

impl<'de, C> BorrowDecode<'de, C> for Uuid {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Self::decode(decoder)
    }
}

impl Encode for Uuid {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.0.minor.encode(encoder)?;
        self.0.major.encode(encoder)
    }
}
