use crate::syntax::ast::{AstNodeIndex, Literal, Type as AstType};
use crate::syntax::tokens::TokenType;
use maplit::hashmap;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::string::String as StdString;
use std::sync::LazyLock;

pub trait TypeSize {
    fn size(&self) -> usize;
}

pub trait InnerTypeEq {
    fn inner_type_eq(&self, other: &Self) -> bool;
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FunctionType {
    pub arguments: Vec<BuiltinType>,
    pub return_type: BuiltinType,
}

impl InnerTypeEq for FunctionType {
    fn inner_type_eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructType {
    pub ast_node_index: AstNodeIndex,
    pub name: StdString,
    pub fields: HashMap<StdString, Type>
}

impl InnerTypeEq for StructType {
    fn inner_type_eq(&self, other: &Self) -> bool {
        self.ast_node_index == other.ast_node_index
    }
}

impl Hash for StructType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ast_node_index.hash(state);
    }
}

impl Display for StructType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "struct {}{{{}}}",
            self.name, 
            self.fields.iter()
                .map(|(field_name, field_type)|
                    format!("{}: {}", field_name, field_type)
                )
                .collect::<Vec<StdString>>()
                .join(", ")
        )
    }
}

impl TypeSize for StructType {
    fn size(&self) -> usize {
        self.fields.iter()
            .map(|(_, f)| f.size())
            .sum()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PointerType {
    pub inner_type: BuiltinType,
    pub mutable: bool,
}

impl PointerType {
    pub fn new(inner_type: BuiltinType, mutable: bool) -> Self {
        Self {
            inner_type,
            mutable,
        }
    }
}

impl InnerTypeEq for PointerType {
    fn inner_type_eq(&self, other: &Self) -> bool {
        self.inner_type.inner_type_eq(&other.inner_type)
    }
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
pub enum BuiltinType {
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
    Pointer(Box<PointerType>),
    Function(Box<FunctionType>),
    Struct(Box<StructType>),
}

impl TypeSize for BuiltinType {
    fn size(&self) -> usize {
        match self {
            Void => 0,
            Null => size_of::<usize>(),
            Bool => size_of::<bool>(),
            I8 | U8 | Char => 1,
            I16 | U16 => 2,
            I32 | U32 | F32 => 4,
            I64 | U64 | F64 => 8,
            Usize | Pointer { .. } | Function(..) => size_of::<usize>(),
            Struct(s) => s.size(),
            not_sized => panic!("Can't get size of {}", not_sized)
        }
    }
}


impl InnerTypeEq for BuiltinType {
    fn inner_type_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Pointer(x), Pointer(y)) => x.inner_type_eq(y),
            (Function(x), Function(y)) => x.inner_type_eq(y),
            (Struct(x), Struct(y)) => x.inner_type_eq(y),
            _ => self == other,
        }
    }
}

static SIMPLE_TYPES_MAP: LazyLock<HashMap<&str, BuiltinType>> = LazyLock::new(
    || hashmap! {
        "void" => Void,
        "bool" => Bool,
        "i8" => I8,
        "i16" => I16,
        "i32" => I32,
        "i64" => I64,
        "u8" => U8,
        "u16" => U16,
        "u32" => U32,
        "u64" => U64,
        "f32" => F32,
        "f64" => F64,
        "usize" => Usize,
        "char" => Char,
        "string" => String,
    }
);

impl BuiltinType {
    pub fn from_literal(literal: &Literal) -> Self {
        match literal {
            Literal::U8 { .. } => U8,
            Literal::U16 { .. } => U16,
            Literal::U32 { .. } => U32,
            Literal::U64 { .. } => U64,
            Literal::I8 { .. } => I8,
            Literal::I16 { .. } => I16,
            Literal::I32 { .. } => I32,
            Literal::I64 { .. } => I64,
            Literal::F32 { .. } => F32,
            Literal::F64 { .. } => F64,
            Literal::Bool { .. } => Bool,
            Literal::Char { .. } => Char,
            Literal::String { .. } | Literal::MultilineString { .. } => String,
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            I8 | I16 | I32 | I64 | U8 | U16 | U32 | U64 => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            F32 | F64 => true,
            _ => false,
        }
    }
    
    pub fn is_bool(&self) -> bool {
        match self {
            Bool => true,
            _ => false,
        }
    }

    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    pub fn is_pointer(&self) -> bool {
        match self {
            Pointer(_) => true,
            _ => false,
        }
    }

    pub fn from_simple_type(simple_type: &AstType) -> Option<BuiltinType> {
        SIMPLE_TYPES_MAP.get(simple_type.name.lexeme.as_str())
            .cloned()
    }

    pub fn from_type_kind(type_kind: &TypeKind) -> BuiltinType {
        match type_kind {
            TypeKind::Simple(simple_type) => BuiltinType::from_simple_type(simple_type).unwrap(),
            TypeKind::Pointer(pointer_type) => {
                let inner_type= Self::from_type_kind(&pointer_type.inner_type);

                inner_type
            },
        }
    }
}

pub fn is_basic_type(name: &str) -> bool {
    static BASIC_TYPES: [&'static str; 15] = [
        "i8", "i16",
        "i32", "i64",
        "u8", "u16",
        "u32", "u64",
        "f32", "f64",
        "usize", "char",
        "string", "cstring",
        "bool",
    ];
    
    BASIC_TYPES.contains(&name)
}


#[derive(Debug, Clone)]
pub struct Type {
    pub ttype: BuiltinType,
    pub mutable: bool,
}


impl TypeSize for Type {
    fn size(&self) -> usize {
        self.ttype.size()
    }
}
impl PartialEq<Self> for Type {
    fn eq(&self, other: &Self) -> bool {
        self.mutable == other.mutable && self.ttype == other.ttype
    }
}

impl Eq for Type {}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{} {}",
            if self.mutable { "mut " } else { "const" },
            self.ttype
        )
    }
}

impl Type {
    pub fn void() -> Self {
        Self {
            ttype: Void,
            mutable: false,
        }
    }
    
    pub fn new(ttype: BuiltinType, mutable: bool) -> Self {
        Self {
            ttype,
            mutable,
        }
    }
    
    pub fn new_pointer(ttype: Type, points_to_mut: bool, mutable: bool) -> Self {
        Self {
            ttype: Pointer(Box::new(PointerType::new(
                ttype.ttype, points_to_mut
            ))),
            mutable
        }
    }

    pub fn from_literal(literal: &Literal) -> Self {
        Self::new(
            BuiltinType::from_literal(literal),
            false
        )
    }

    pub fn from_type_annotation(type_annotation: &TypeAnnotation) -> Self {
        let builtin_type = BuiltinType::from_type_kind(&type_annotation.kind);


        Self::new(
            builtin_type, type_annotation.is_mut
        )
    }
}

impl InnerTypeEq for Type {
    fn inner_type_eq(&self, other: &Type) -> bool {
        self.ttype.inner_type_eq(&other.ttype)
    }
}
impl Display for BuiltinType {
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
            String => write!(f, "const char*"),
            Pointer(pointer) =>
                write!(
                    f, "{}{}*",
                    if pointer.mutable { "" } else { "const" },
                    pointer.inner_type
            ),
            Function(function) => write!(f, "{}", function),
            Struct(name) => write!(f, "struct {}", name),
        }
    }
}


use crate::syntax::ast::{TypeAnnotation, TypeKind};
use BuiltinType::*;

type TypeBinaryOpMap = HashMap<(&'static BuiltinType, &'static BuiltinType), BuiltinType>;

pub static PLUS_OP_MAP: LazyLock<TypeBinaryOpMap> = LazyLock::new(
    || hashmap! {
        (&I8, &I8) => I8,
        (&I16, &I16) => I16,
        (&I32, &I32) => I64,
        (&I64, &I64) => I64,
        (&U8, &U8) => U8,
        (&U16, &U16) => U16,
        (&U32, &U32) => U32,
        (&U64, &U64) => U64,
        (&F32, &F32) => F32,
        (&F64, &F64) => F64,
        (&Usize, &Usize) => Usize,
        (&Char, &Char) => Char,
        (&String, &String) => String,
    }
);

pub static SUB_OP_MAP: LazyLock<TypeBinaryOpMap> = LazyLock::new(
    || hashmap! {
        (&I8, &I8) => I8,
        (&I16, &I16) => I16,
        (&I32, &I32) => I64,
        (&I64, &I64) => I64,
        (&U8, &U8) => U8,
        (&U16, &U16) => U16,
        (&U32, &U32) => U32,
        (&U64, &U64) => U64,
        (&F32, &F32) => F32,
        (&F64, &F64) => F64,
        (&Usize, &Usize) => Usize,
        (&Char, &Char) => Char,
    }
);

pub static MUL_OP_MAP: LazyLock<TypeBinaryOpMap> = LazyLock::new(
    || hashmap! {
        (&I8, &I8) => I8,
        (&I16, &I16) => I16,
        (&I32, &I32) => I64,
        (&I64, &I64) => I64,
        (&U8, &U8) => U8,
        (&U16, &U16) => U16,
        (&U32, &U32) => U32,
        (&U64, &U64) => U64,
        (&F32, &F32) => F32,
        (&F64, &F64) => F64,
        (&Usize, &String) => String,
        (&String, &Usize) => String,
    }
);

pub static DIV_OP_MAP: LazyLock<TypeBinaryOpMap> = LazyLock::new(
    || hashmap! {
        (&I8, &I8) => I8,
        (&I16, &I16) => I16,
        (&I32, &I32) => I64,
        (&I64, &I64) => I64,
        (&U8, &U8) => U8,
        (&U16, &U16) => U16,
        (&U32, &U32) => U32,
        (&U64, &U64) => U64,
        (&F32, &F32) => F32,
        (&F64, &F64) => F64,
        (&Usize, &Usize) => Usize,
    }
);

pub static UNARY_MINUS_OP_MAP: LazyLock<HashMap<BuiltinType, BuiltinType>> = LazyLock::new(
    || hashmap! {
        I8 => I8,
        I16 => I16,
        I32 => I32,
        I64 => I64,
        F32 => F32,
        F64 => F64,
    }
);


pub static LOGICAL_OP_MAP: LazyLock<TypeBinaryOpMap> = LazyLock::new(
    || hashmap! {
        (&Bool, &Bool) => Bool,
    }
);


pub static LOGICAL_UNARY_MAP: LazyLock<HashMap<BuiltinType, BuiltinType>> = LazyLock::new(
    || hashmap! {
        Bool => Bool,
    }
);

pub static COMPARISON_OP_MAP: LazyLock<TypeBinaryOpMap> = LazyLock::new(
  || hashmap! {
      (&I8, &I8) => Bool,
      (&I16, &I16) => Bool,
      (&I32, &I32) => Bool,
      (&I64, &I64) => Bool,
      (&U8, &U8) => Bool,
      (&U16, &U16) => Bool,
      (&U32, &U32) => Bool,
      (&U64, &U64) => Bool,
      (&F32, &F32) => Bool,
      (&F64, &F64) => Bool,
      (&Usize, &Usize) => Bool,
      (&Char, &Char) => Bool,
      (&Bool, &Bool) => Bool,
      (&Null, &Null) => Bool,
      (&String, &String) => Bool,
  }
);


pub fn match_pointer_binary_op_comm(lhs: &Type, rhs: &Type) -> Option<Type> {
    if !(lhs.ttype.is_integer() && rhs.ttype.is_pointer()) ||
        !(rhs.ttype.is_integer() && lhs.ttype.is_pointer()) {
        return None;
    }

    Some(lhs.clone())
}

pub fn match_pointer_binary_op(lhs: &Type, rhs: &Type) -> Option<Type> {
    if !lhs.ttype.is_pointer() || !rhs.ttype.is_integer() {
        return None;
    }

    Some(lhs.clone())
}

pub fn match_pointer_binary_comparison(
    lhs: &Type, rhs: &Type
) -> Option<Type> {

    if let (
        Pointer(p_lhs),
        Pointer(p_rhs),
    ) = (&lhs.ttype, &rhs.ttype)
    {
        if p_lhs.inner_type_eq(p_rhs) {
            Some(Type::new(Bool, false))
        } else {
            None
        }
    } else {
        None
    }
}

fn match_binary_op_map(
    map: &TypeBinaryOpMap,
    lhs: &Type,
    rhs: &Type,
) -> Option<Type>
{
    if let Some(result_type) = map.get(&(&lhs.ttype, &rhs.ttype)) {
        Some(Type::new(result_type.clone(), lhs.mutable))
    } else {
        None
    }
}

fn match_binary_op_comm_map(
    map: &TypeBinaryOpMap,
    lhs: &Type,
    rhs: &Type,
) -> Option<Type>
{
    if let Some(result_type) = map.get(&(&lhs.ttype, &rhs.ttype)) {
        Some(Type::new(result_type.clone(), lhs.mutable))
    } else if let Some(result_type) = map.get(&(&rhs.ttype, &lhs.ttype)) {
        Some(Type::new(result_type.clone(), rhs.mutable))
    } else {
        None
    }
}

fn match_binary_op_impl(
    map: &TypeBinaryOpMap, lhs: &Type, rhs: &Type
) -> Option<Type> {
    match_binary_op_map(map, lhs, rhs)
        .or_else(|| match_pointer_binary_op(&lhs, &rhs))
}

fn match_binary_op_impl_comm(
    map: &TypeBinaryOpMap, lhs: &Type, rhs: &Type
) -> Option<Type> {
    match_binary_op_comm_map(map, lhs, rhs)
        .or_else(|| match_pointer_binary_op_comm(lhs, rhs))
}

pub fn match_binary_op(op: TokenType, lhs: &Type, rhs: &Type)
    -> Option<Type>
{
    match op {
        TokenType::Plus =>
            match_binary_op_impl_comm(&PLUS_OP_MAP, lhs, rhs),
        TokenType::Minus =>
            match_binary_op_impl_comm(&SUB_OP_MAP, lhs, rhs),
        TokenType::Star => match_binary_op_comm_map(&MUL_OP_MAP, lhs, rhs),
        TokenType::Slash => match_binary_op_comm_map(&DIV_OP_MAP, lhs, rhs),
        TokenType::EqualsEquals | TokenType::NotEquals |
        TokenType::Less | TokenType::LessEqual |
        TokenType::Greater | TokenType::GreaterEqual =>
            match_pointer_binary_comparison(&lhs, &rhs)
                .or_else(||
                    COMPARISON_OP_MAP.get(&(&lhs.ttype, &rhs.ttype)).cloned()
                    .map(|bt| Type::new(bt, false))
                ),
        TokenType::LogicalOr | TokenType::LogicalAnd => match_binary_op_comm_map(
            &LOGICAL_OP_MAP, lhs, rhs
        ),
        _ => None
    }
}

pub fn match_inplace_assignment_op(op: TokenType, lhs: &Type, rhs: &Type)
    -> Option<Type>
{
    match op {
        TokenType::PlusEquals => match_binary_op_impl(&PLUS_OP_MAP, lhs, rhs),
        TokenType::MinusEquals => match_binary_op_impl(&SUB_OP_MAP, lhs, rhs),
        TokenType::StarEquals => match_binary_op_impl(&MUL_OP_MAP, lhs, rhs),
        TokenType::SlashEquals => match_binary_op_impl(&DIV_OP_MAP, lhs, rhs),
        TokenType::BinaryAndEquals | TokenType::BinaryOrEquals =>
            match_binary_op_impl(&LOGICAL_OP_MAP, lhs, rhs),
        // TokenType::BinaryInvertEquals => {},
        // TokenType::BinaryXorEquals => {},
        // TokenType::BinaryShiftLeftEquals => {},
        // TokenType::BinaryShiftRightEquals => {},
        _ => None
    }
}

pub fn match_unary_op(op: TokenType, rhs: &Type) -> Option<Type> {
    match op {
        TokenType::Minus => UNARY_MINUS_OP_MAP.get(&rhs.ttype)
            .map(|builtin_type| Type::new(
                builtin_type.clone(),
                rhs.mutable
            )),
        TokenType::Star =>
            if let Pointer(pointer) = &rhs.ttype {
                Some(Type::new(
                    pointer.inner_type.clone(),
                    pointer.mutable
                ))
            } else {
                None
            }
        TokenType::BinaryAnd => Some(Type::new(
            Pointer(Box::new(PointerType::new(rhs.ttype.clone(), false))),
            false
        )),
        TokenType::MutRef => Some(Type::new(
            Pointer(Box::new(PointerType::new(rhs.ttype.clone(), true))),
            true,
        )),
        TokenType::LogicalNot => if rhs.ttype.is_bool() {
            Some(Type::new(Bool, rhs.mutable))
        } else {
            None
        },
        _ => None
    }
}

pub fn verify_cast_operator(lhs: &Type, target_type: &Type) -> bool {
    if (lhs.ttype == Char || lhs.ttype == Bool || lhs.ttype.is_numeric()) &&
        (target_type.ttype == Char || target_type.ttype == Bool || target_type.ttype.is_numeric())
    {
        return true;
    }

    let Pointer(lhs_pointer) = &lhs.ttype else {
        return false;
    };

    if target_type.ttype.is_integer() {
        return true;
    }

    let Pointer(target_pointer) = &target_type.ttype
    else {
        return false;
    };

    match target_pointer.inner_type {
        Void => true,
        _ => lhs_pointer.inner_type.size() == target_pointer.inner_type.size()
    }
}
