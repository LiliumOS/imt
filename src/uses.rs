use std::num::NonZero;

use bincode::{Decode, Encode};

use crate::{
    attr::{Attribute, AttributeTarget, AttributeTargetKind},
    uuid::Uuid,
};

#[derive(Clone, Debug, Encode, Decode)]
pub enum Type {
    Named(String, Option<Vec<Type>>),
    Param(u32, Option<Box<Type>>),
    Int(IntType),
    Pointer(PointerKind, Box<Type>),
    Func(Signature),
    Void,
    Never,
    Byte,
    Char(IntType),
    Array(Box<ArrayType>),
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct ArrayType {
    pub base: Type,
    pub len: Expr,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Encode, Decode)]
pub struct IntType {
    pub signed: bool,
    pub bits: IntBits,
}

macro_rules! nzlit {
    ($lit:literal) => {
        const {
            let __val = $lit;
            assert!(__val != 0);
            unsafe { core::num::NonZero::new_unchecked(__val) }
        }
    };
}

#[allow(non_upper_case_globals)]
impl IntType {
    pub const i8: IntType = IntType {
        signed: true,
        bits: IntBits::Bits(nzlit!(8)),
    };
    pub const i16: IntType = IntType {
        signed: true,
        bits: IntBits::Bits(nzlit!(16)),
    };
    pub const i32: IntType = IntType {
        signed: true,
        bits: IntBits::Bits(nzlit!(32)),
    };
    pub const i64: IntType = IntType {
        signed: true,
        bits: IntBits::Bits(nzlit!(64)),
    };
    pub const i128: IntType = IntType {
        signed: true,
        bits: IntBits::Bits(nzlit!(128)),
    };
    pub const ilong: IntType = IntType {
        signed: true,
        bits: IntBits::Long,
    };

    pub const u8: IntType = IntType {
        signed: false,
        bits: IntBits::Bits(nzlit!(8)),
    };
    pub const u16: IntType = IntType {
        signed: false,
        bits: IntBits::Bits(nzlit!(16)),
    };
    pub const u32: IntType = IntType {
        signed: false,
        bits: IntBits::Bits(nzlit!(32)),
    };
    pub const u64: IntType = IntType {
        signed: false,
        bits: IntBits::Bits(nzlit!(64)),
    };
    pub const u128: IntType = IntType {
        signed: false,
        bits: IntBits::Bits(nzlit!(128)),
    };
    pub const ulong: IntType = IntType {
        signed: false,
        bits: IntBits::Long,
    };
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Encode, Decode)]
pub enum IntBits {
    Long,
    Bits(NonZero<u8>),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Encode, Decode)]
pub enum PointerKind {
    Const,
    Mut,
    Special(Uuid),
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Signature {
    pub params: Vec<Param>,
    pub retty: Box<Type>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Param {
    pub attrs: Vec<Attribute<Param>>,
    pub name: Option<String>,
    pub ty: Type,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum Expr {
    IntLiteral(IntType, u128),
    UuidLiteral(Uuid),
    StringLiteral(String),
    Const(String),
    BinOp(BinaryOp, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    SpecialConstant(SpecialConst),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Encode, Decode)]
#[non_exhaustive]
pub enum SpecialConst {
    SizeofPointer,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Encode, Decode)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Encode, Decode)]
pub enum UnaryOp {
    Not,
    Neg,
}
