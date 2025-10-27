use bincode::{Decode, Encode};

use crate::{
    attr::Attribute,
    uses::{Expr, Signature, Type},
};

#[derive(Clone, Debug, Encode, Decode)]
pub struct Value {
    pub name: String,
    pub body: ValueBody,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum ValueBody {
    Const(Const),
    Function(Function),
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Const {
    pub attrs: Vec<Attribute<Const>>,
    pub ty: Type,
    pub val: Expr,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Function {
    pub attrs: Vec<Attribute<Function>>,
    pub signature: Signature,
}
