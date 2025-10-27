use bincode::{Decode, Encode};

use crate::{
    attr::{Attribute, AttributeTarget, AttributeTargetKind},
    header::Header,
    tydef::TypeDef,
    uuid::Uuid,
    value::Value,
};

#[derive(Clone, Debug, Encode, Decode)]
pub struct File {
    pub header: Header,
    pub file_id: Uuid,
    pub attributes: Vec<Attribute<File>>,
    pub uses: Vec<UseItem>,
    pub types: Vec<TypeDef>,
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct UseItem {
    pub attrs: Vec<Attribute<UseItem>>,
    pub path: Vec<String>,
}
