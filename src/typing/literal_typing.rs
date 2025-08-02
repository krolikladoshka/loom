use crate::syntax::tokens::TokenType;
use maplit::hashmap;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::string::String as StdString;
use std::sync::LazyLock;


#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FunctionType {
    pub arguments: Vec<NumericType>,
    pub return_type: NumericType,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct PointerType {
    pub inner_type: NumericType,
    pub mutable: bool,
}

impl Debug for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let args = self.arguments.iter()
            .map(|arg| format!("{}", arg))
            .collect::<Vec<StdString>>()
            .join(", ");

        write!(f, "FunctionType[{}]", args)
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


// impl Hash for FunctionType {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.arguments
//             .for_each(|lt| lt.hash(state));
//         self.return_type.hash(state);
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumericType {
    Void,
    Null,
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Usize,
    Char,
    String,
    Pointer(Box<NumericType>),
    Function(Box<FunctionType>),
}

pub struct Type {
    pub ttype: NumericType,
    pub mutable: bool,
}

impl Display for NumericType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Void => write!(f, "void"),
            Null => write!(f, "null"),
            Bool => write!(f, "bool"),
            I8 => write!(f, "i8"),
            I16 => write!(f, "i16"),
            I32 => write!(f, "i32"),
            I64 => write!(f, "i64"),
            U8 => write!(f, "u8"),
            U16 => write!(f, "u16"),
            U32 => write!(f, "u32"),
            U64 => write!(f, "u64"),
            F32 => write!(f, "f32"),
            F64 => write!(f, "f64"),
            Usize => write!(f, "usize"),
            Char => write!(f, "char"),
            String => write!(f, "char*"),
            Pointer(inner_type) => write!(f, "{}*", inner_type),
            Function(function) => write!(f, "{}", function),
        }
    }
}


use NumericType::*;

pub static PLUS_OP_MAP: LazyLock<HashMap<(NumericType, NumericType), NumericType>> = LazyLock::new(
    || hashmap! {
        (I8, I8) => I8,
        (I16, I16) => I16,
        (I32, I32) => I64,
        (I64, I64) => I64,
        (U8, U8) => U8,
        (U16, U16) => U16,
        (U32, U32) => U32,
        (U64, U64) => U64,
        (F32, F32) => F32,
        (F64, F64) => F64,
        (Usize, Usize) => Usize,
        (String, String) => String,
    }
);

pub static SUB_OP_MAP: LazyLock<HashMap<(NumericType, NumericType), NumericType>> = LazyLock::new(
    || hashmap! {
        (I8, I8) => I8,
        (I16, I16) => I16,
        (I32, I32) => I64,
        (I64, I64) => I64,
        (U8, U8) => U8,
        (U16, U16) => U16,
        (U32, U32) => U32,
        (U64, U64) => U64,
        (F32, F32) => F32,
        (F64, F64) => F64,
        (Usize, Usize) => Usize,
        (String, String) => String,
    }
);

pub static MUL_OP_MAP: LazyLock<HashMap<(NumericType, NumericType), NumericType>> = LazyLock::new(
    || hashmap! {
        (I8, I8) => I8,
        (I16, I16) => I16,
        (I32, I32) => I64,
        (I64, I64) => I64,
        (U8, U8) => U8,
        (U16, U16) => U16,
        (U32, U32) => U32,
        (U64, U64) => U64,
        (F32, F32) => F32,
        (F64, F64) => F64,
        (Usize, String) => String,
    }
);

pub static DIV_OP_MAP: LazyLock<HashMap<(NumericType, NumericType), NumericType>> = LazyLock::new(
    || hashmap! {
        (I8, I8) => I8,
        (I16, I16) => I16,
        (I32, I32) => I64,
        (I64, I64) => I64,
        (U8, U8) => U8,
        (U16, U16) => U16,
        (U32, U32) => U32,
        (U64, U64) => U64,
        (F32, F32) => F32,
        (F64, F64) => F64,
        (Usize, Usize) => Usize,
    }
);

pub static UNARY_MINUS_OP_MAP: LazyLock<HashMap<(NumericType), NumericType>> = LazyLock::new(
    || hashmap! {
        I8 => I8,
        I16 => I16,
        I32 => I32,
        I64 => I64,
        F32 => F32,
        F64 => F64,
    }
);

pub static LOGICAL_OP_MAP: LazyLock<HashMap<(NumericType, NumericType), NumericType>> = LazyLock::new(
    || hashmap! {
        (Bool, Bool) => Bool,
    }
);


pub static LOGICAL_UNARY_MAP: LazyLock<HashMap<NumericType, NumericType>> = LazyLock::new(
    || hashmap! {
        Bool => Bool,
    }
);

pub static COMPARISON_OP_MAP: LazyLock<HashMap<(NumericType, NumericType), NumericType>> = LazyLock::new(
  || hashmap! {
      (I8, I8) => Bool,
      (I16, I16) => Bool,
      (I32, I32) => Bool,
      (I64, I64) => Bool,
      (U8, U8) => Bool,
      (U16, U16) => Bool,
      (U32, U32) => Bool,
      (U64, U64) => Bool,
      (F32, F32) => Bool,
      (F64, F64) => Bool,
      (Usize, Usize) => Bool,
      (Char, Char) => Bool,
      (Bool, Bool) => Bool,
      (Null, Null) => Bool,
      (String, String) => Bool,
  }
);

pub fn match_binary_op(op: TokenType, lhs: NumericType, rhs: NumericType) -> Option<NumericType> {
    match op {
        TokenType::Plus => PLUS_OP_MAP.get(&(lhs, rhs)).cloned(),
        TokenType::Minus => SUB_OP_MAP.get(&(lhs, rhs)).cloned(),
        TokenType::Star => MUL_OP_MAP.get(&(lhs, rhs)).cloned(),
        TokenType::Slash => DIV_OP_MAP.get(&(lhs, rhs)).cloned(),
        TokenType::EqualsEquals | TokenType::NotEquals |
        TokenType::Less | TokenType::LessEqual |
        TokenType::Greater | TokenType::GreaterEqual => COMPARISON_OP_MAP
            .get(&(lhs, rhs)).cloned(),
        TokenType::LogicalOr | TokenType::LogicalAnd => LOGICAL_OP_MAP
            .get(&(lhs, rhs)).cloned(),
        _ => None
    }
}

pub fn map_unary(op: TokenType, rhs: NumericType) -> Option<NumericType> {
    match op {
        TokenType::Minus => UNARY_MINUS_OP_MAP.get(&rhs).cloned(),
        TokenType::Star => todo!("Dereference operator"),
        _ => None
    }
}

