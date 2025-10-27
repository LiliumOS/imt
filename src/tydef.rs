use bincode::{Decode, Encode};

use crate::{
    attr::{Attribute, AttributeTarget, AttributeTargetKind},
    uses::{Expr, IntType, Type},
};

#[derive(Clone, Debug, Encode, Decode)]
pub struct TypeDef {
    pub name: String,
    pub num_params: u32,
    pub body: TypeDefBody,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum TypeDefBody {
    Alias(TypeAlias),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct TypeAlias {
    pub attrs: Vec<Attribute<TypeAlias>>,
    pub alias: Type,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Struct {
    pub attrs: Vec<Attribute<Struct>>,
    pub body: StructBody,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum StructBody {
    Fields(StructFields),
    Opaque(Option<Type>),
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct StructFields {
    pub field: Vec<Field>,
    pub pad: Option<Type>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Field {
    pub attrs: Vec<Attribute<Field>>,
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Union {
    pub attrs: Vec<Attribute<Union>>,
    pub fields: StructFields,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Enum {
    pub attrs: Vec<Attribute<Enum>>,
    pub underlying: IntType,
    pub variants: Vec<Variant>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Variant {
    pub attrs: Vec<Attribute<Variant>>,
    pub name: String,
    pub discrim: Expr,
}
