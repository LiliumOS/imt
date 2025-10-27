use bincode::{BorrowDecode, Decode, Encode, de::read::Reader};

use crate::uuid::Uuid;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct MagicNumber;

pub const MAGIC: [u8; 6] = *b"\xFEIMTDB";

impl<C> Decode<C> for MagicNumber {
    fn decode<D: bincode::de::Decoder<Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let magic: [u8; 6] = Decode::decode(decoder)?;

        if magic != MAGIC {
            return Err(bincode::error::DecodeError::Other("Invalid Magic Number"));
        }
        Ok(MagicNumber)
    }
}

impl<'de, C> BorrowDecode<'de, C> for MagicNumber {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Self::decode(decoder)
    }
}

impl Encode for MagicNumber {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        MAGIC.encode(encoder)
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Encode, Decode)]
pub struct Version(u16);

impl Version {
    pub const fn new(major: u16, minor: u16) -> Self {
        assert!(major < 128, "Major version must be less than 128");
        assert!(minor < 512, "Minor Version must be less than 512");

        Self((major << 9) | minor)
    }

    pub const fn parse(v: &str) -> Self {
        let mut major = 0;
        let mut minor = 0;
        assert!(
            v.len() > 2,
            "Must have a non-empty major and minor component"
        );

        let buf = v.as_bytes();

        let mut n = 0;

        while n < buf.len() {
            match buf[n] {
                b @ b'0'..=b'9' => {
                    major *= 10;
                    major += (b - b'0') as u16;
                    n += 1;
                }
                b'.' => {
                    n += 1;
                    break;
                }
                _ => {
                    panic!("Expected a version of the form <major>.<minor>")
                }
            }
        }

        while n < buf.len() {
            match buf[n] {
                b @ b'0'..=b'9' => {
                    minor *= 10;
                    minor += (b - b'0') as u16;
                    n += 1;
                }
                _ => {
                    panic!("Expected a version of the form <major>.<minor>")
                }
            }
        }

        Self::new(major, minor)
    }

    pub const fn major(self) -> u16 {
        self.0 >> 9
    }

    pub const fn minor(self) -> u16 {
        self.0 & 511
    }

    pub const fn is_compatible(self, other: Version) -> bool {
        self.major() == other.major()
            && self.minor() <= other.minor()
            && (self.major() != 0 || self.minor() == other.minor())
    }
}

impl core::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Version")
            .field("major", &self.major())
            .field("minor", &self.minor())
            .finish()
    }
}

impl core::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}.{}", self.major(), self.minor()))
    }
}

pub const CURRENT_VERSION: Version = Version::parse(core::concat!(
    core::env!("CARGO_PKG_VERSION_MAJOR"),
    ".",
    core::env!("CARGO_PKG_VERSION_MINOR")
));

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Encode, Decode)]
pub struct Header {
    pub magic: MagicNumber,
    pub version: Version,
}

impl Header {
    pub const CURRENT: Header = Header {
        magic: MagicNumber,
        version: CURRENT_VERSION,
    };
}
