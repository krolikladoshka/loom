use std::fmt::Pointer;
use crate::syntax::lexer::Token;
use crate::syntax::traits::{partial_eq, partial_eq_all, PartialTreeEq};

#[derive(Debug, Clone)]
pub struct Identifier {
    pub name: Token,
}

pub struct Variable {

}

// #[derive(Debug, Clone)]
// pub struct Literal {
//
// }

#[derive(Debug, Clone)]
pub enum Literal {
    U8 {
        token: Token,
        value: u8,
    },
    U16 {
        token: Token,
        value: u16,
    },
    U32 {
        token: Token,
        value: u32,
    },
    U64 {
        token: Token,
        value: u64,
    },
    I8 {
        token: Token,
        value: i8,
    },
    I16 {
        token: Token,
        value: i16,
    },
    I32 {
        token: Token,
        value: i32,
    },
    I64 {
        token: Token,
        value: i64,
    },
    F32 {
        token: Token,
        value: f32,
    },
    F64 {
        token: Token,
        value: f64,
    },
    Bool {
        token: Token,
        value: bool,
    },
    Char {
        token: Token,
        value: char,
    },
    String {
        token: Token,
        value: String,
    },
    MultilineString {
        token: Token,
        value: String,
    },
}

#[derive(Debug, Clone)]
pub struct Type {
    pub name: Token,
}
#[derive(Debug, Clone)]
pub struct PointerAnnotation {
    pub inner_type: Box<TypeKind>,
    pub points_to_mut: bool,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Simple(Type),
    Pointer(PointerAnnotation),
}

#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    // pub name: Token,
    pub kind: TypeKind,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct TypedDeclaration {
    pub name: Token,
    pub declared_type: TypeAnnotation,
}

pub struct SizedTypedDeclaration {
    name: Token,
    declared_type: Type,
    size: usize,
}


#[derive(Debug)]
pub enum Expression {
    Grouping {
        token: Token,
        expression: Box<Expression>,
    },
    Literal(Literal),
    Identifier {
        name: Token,
    },
    MethodCall(),
    DotSet {},
    ArrowSet {},
    DotAccess {
        object: Box<Expression>,
        name: Token,
    },
    ArrowAccess {
        pointer: Box<Expression>,
        name: Token,
    },
    Call {
        token: Token,
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    ArraySlice {
        token: Token,
        array_expression: Box<Expression>,
        slice_expression: Box<Expression>,
    },
    Unary {
        token: Token,
        operator: Token,
        expression: Box<Expression>,
    },
    Cast {
        token: Token,
        left: Box<Expression>,
        target_type: TypeAnnotation,
        is_reinterpret_cast: bool,
    },
    Binary {
        // token: Token,
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    // LogicalBinary {
    //     token: Token,
    //     left: Box<Expression>,
    //     operator: Token,
    //     right: Box<Expression>,
    // },
    Range {
        token: Token,
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },
    InplaceAssignment {
        token: Token,
        lhs: Box<Expression>,
        operator: Token,
        rhs: Box<Expression>,
    },
    Assignment {
        token: Token,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    IfElseExpression {
        token: Token,
        condition: Box<Expression>,
        then_branch: Box<Statement>,
        else_branch: Box<Statement>,
    },
    Block {
        token: Token,
        statements: Vec<Statement>,
        return_expression: Option<Box<Expression>>,
    },
    SelfExpression {
        token: Token,
    },
    FnExpression {
        token: Token,
        function: Function,
    },
    StructInitializer {
        token: Token,
        struct_name: Token,
        field_initializers: Vec<(Token, Expression)>,
    }
}

impl Expression {

}

pub struct Struct {

}

#[derive(Debug)]
pub struct Function {
    pub name: Token,
    pub arguments: Vec<TypedDeclaration>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct Method {
    pub name: Token,
    pub bound_type: Token,
    pub is_mut_self: bool,
    pub arguments: Vec<TypedDeclaration>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub enum ImplFunction {
    Function(Function),
    Method(Method),
}

pub struct EnumVariant {}

#[derive(Debug)]
pub enum Statement {
    EmptyStatement { //++
        semicolon_token: Token,
    },
    LetStatement { //+-
        token: Token,
        name: Token,
        variable_type: Option<TypeAnnotation>,
        initializer: Option<Box<Expression>>,
        is_mut: bool,
    },
    StaticStatement { //+-
        token: Token,
        name: Token,
        variable_type: TypeAnnotation,
        initializer: Box<Expression>,
        is_mut: bool,
    },
    ConstStatement { //+-
        token: Token,
        name: Token,
        variable_type: TypeAnnotation,
        initializer: Box<Expression>,
    },
    ExpressionStatement { //++
        expression: Box<Expression>,
    },
    WhileStatement { //++
        token: Token,
        condition: Box<Expression>,
        body: Vec<Statement>,
    },
    BreakStatement { //+
        token: Token,
        loop_key: Option<Token>,
    },
    ContinueStatement { //+
        token: Token,
        loop_key: Option<Identifier>
    },
    FnStatement { //+
        token: Token,
        function: ImplFunction,
    },
    ReturnStatement { //+
        token: Token,
        expression: Option<Box<Expression>>
    },
    DeferStatement {
        token: Token,
        call_expression: Box<Expression>,
        to_closest_block: bool
    },
    StructStatement {
        token: Token,
        name: Token,
        fields: Vec<TypedDeclaration>,
    },
    // UnionStructStatement {
    //     token: Token,
    //     name: Token,
    //     fields: Vec<SizedTypedDeclaration>
    // },
    // EnumStructStatement {
    //     token: Token,
    //     name: Token,
    //     variants: Vec<EnumVariant>
    // },

    ImplStatement {
        token: Token,
        implemented_type: Token,
        top_level_statements: Vec<Statement>,
        functions: Vec<ImplFunction>,
    },
    IfElseStatement {
        token: Token,
        condition: Box<Expression>,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    }
    
}

pub enum Ast {
    Expression(Expression),
    Statement(Statement)
}