use bincode::{Decode, Encode};

use crate::{header::Version, uuid::Uuid};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default, Encode, Decode)]
pub enum SafetyHint {
    #[default]
    NoHint,
    Safe,
    Unsafe,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default, Encode, Decode)]
pub struct OptionType {
    pub option: Uuid,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default, Encode, Decode)]
pub struct PolymorphicOption;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, Encode, Decode)]
pub struct ItemDoc {
    pub doc_lines: Vec<String>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, Encode, Decode)]
pub struct SubsystemDescriptor {
    pub subsys_id: Uuid,
    pub subsys_index: Option<u32>,
    pub version: Version,
    pub max_sysfn: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, Encode, Decode)]
pub struct SystemFunction {
    pub function_id: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, Encode, Decode)]
pub struct ExportInline;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, Encode, Decode)]
#[non_exhaustive]
pub enum DefinesBuiltinTypes {
    #[default]
    None,
    Handle,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, Encode, Decode)]
pub struct ToolComment {
    pub comment: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, Encode, Decode)]
pub struct Align {
    pub alignment: u128,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, Encode, Decode)]
pub struct Synthetic;
