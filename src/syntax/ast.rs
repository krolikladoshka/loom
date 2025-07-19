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

impl PartialTreeEq for Type {
    type Other = Type;

    fn partial_eq(&self, other: &Self::Other) -> bool {
        self.name.partial_eq(&other.name)
    }
}

#[derive(Debug, Clone)]
pub struct PointerAnnotation {
    pub inner_type: Box<TypeKind>,
    pub points_to_mut: bool,
}

impl PartialTreeEq for PointerAnnotation {
    type Other = PointerAnnotation;

    fn partial_eq(&self, other: &Self::Other) -> bool {
        self.points_to_mut == other.points_to_mut &&
            self.inner_type.partial_eq(&other.inner_type)
    }
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Simple(Type),
    Pointer(PointerAnnotation),
}

impl PartialTreeEq for TypeKind {
    type Other = TypeKind;

    fn partial_eq(&self, other: &Self::Other) -> bool {
        match (self, other) {
            (
                Self::Pointer(a),
                Self::Pointer(b)
            ) => a.partial_eq(b),
            (Self::Simple(a), Self::Simple(b)) => a.partial_eq(b),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    // pub name: Token,
    pub kind: TypeKind,
    pub is_mut: bool,
}

impl PartialTreeEq for TypeAnnotation {
    type Other = TypeAnnotation;

    fn partial_eq(&self, other: &TypeAnnotation) -> bool {
        self.is_mut == other.is_mut &&
            self.kind.partial_eq(&other.kind)
        // self.name.partial_eq(&other.name)
    }
}

#[derive(Debug, Clone)]
pub struct TypedDeclaration {
    pub name: Token,
    pub declared_type: TypeAnnotation,
}

impl PartialTreeEq for TypedDeclaration {
    type Other = TypedDeclaration;

    fn partial_eq(&self, other: &TypedDeclaration) -> bool {
        self.name.partial_eq(&other.name) &&
            self.declared_type.partial_eq(&other.declared_type)
    }
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
    DotAccess(),
    Call(),
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
        target_type: Token,
    },
    ReinterpretCast {
        token: Token,
        left: Box<Expression>,
        target_type: Type,
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

impl PartialTreeEq for Function {
    type Other = Function;

    fn partial_eq(&self, other: &Self::Other) -> bool {
        if !self.name.partial_eq(&other.name) {
            return false;
        }

        if !(self.return_type.is_some() && other.return_type.is_some()) &&
            !(self.return_type.is_none() && other.return_type.is_none()) {
            return false;
        }

        if self.return_type.is_some() {
            return self.return_type.as_ref().unwrap()
                .partial_eq(&other.return_type.as_ref().unwrap())
        }
        
        partial_eq_all(&self.arguments, &other.arguments) &&
            partial_eq_all(&self.body, &other.body)
    }
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

impl PartialTreeEq for Method {
    type Other = Method;

    fn partial_eq(&self, other: &Self::Other) -> bool {
        if !self.bound_type.partial_eq(&other.bound_type) {
            return false;
        }
        if !(self.return_type.is_some() && other.return_type.is_some()) &&
            !(self.return_type.is_none() && other.return_type.is_none()) {
            return false;
        }

        if self.return_type.is_some() {
            return self.return_type.as_ref().unwrap()
                .partial_eq(&other.return_type.as_ref().unwrap())
        }

        partial_eq_all(&self.arguments, &other.arguments) &&
            partial_eq_all(&self.body, &other.body)
    }
}

#[derive(Debug)]
pub enum ImplFunction {
    Function(Function),
    Method(Method),
}

impl PartialTreeEq for ImplFunction {
    type Other = ImplFunction;

    fn partial_eq(&self, other: &Self::Other) -> bool {
        match (self, other) {
            (
                Self::Function(a),
                Self::Function(b),
            ) => a.partial_eq(b),
            (
                Self::Method(a),
                Self::Method(b),
            ) => a.partial_eq(b),
            _ => false,
        }
    }
}

pub struct EnumVariant {}

#[derive(Debug)]
pub enum Statement {
    EmptyStatement { //+
        semicolon_token: Token,
    },
    LetStatement {
        token: Token,
        name: Token,
        variable_type: Option<TypeAnnotation>,
        initializer: Option<Box<Expression>>,
    },
    StaticStatement { //+
        token: Token,
        name: Token,
        variable_type: TypeAnnotation,
        initializer: Box<Expression>,
    },
    ConstStatement { //+
        token: Token,
        name: Token,
        variable_type: TypeAnnotation,
        initializer: Box<Expression>,
    },
    ExpressionStatement { //+
        expression: Box<Expression>,
    },
    WhileStatement { //+
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
    }
}

impl PartialTreeEq for Statement {
    type Other = Statement;

    fn partial_eq(&self, other: &Statement) -> bool {
        use Statement::*;

        match (self, other) {
            (
                EmptyStatement { semicolon_token: a },
                EmptyStatement { semicolon_token: b}
            ) => a.partial_eq(b),
            (
                StructStatement {
                    token: ta,
                    name: na,
                    fields: fa
                },
                StructStatement {
                    token: tb,
                    name: nb,
                    fields: fb
                }
            ) => {
                if !ta.partial_eq(tb) || !na.partial_eq(nb) {
                    return false;
                }
                
                partial_eq_all(fa, fb)
            },
            (
                FnStatement {
                    token: ta, function: fa
                },
                FnStatement {
                    token: tb, function: fb,
                },
            ) =>
                ta.partial_eq(tb) && fa.partial_eq(fb),
            (
                ImplStatement {
                    token: ta,
                    implemented_type: ita,
                    top_level_statements: tlsa,
                    functions: fa,
                },
                ImplStatement {
                    token: tb,
                    implemented_type: itb,
                    top_level_statements: tlsb,
                    functions: fb,
                }
            ) => {
                if !(partial_eq(ta, tb) && partial_eq(ita, itb)) {
                    return false;
                }

                partial_eq_all(tlsa, tlsb) && partial_eq_all(fa, fb)
            },
            _ => false
        }
    }
}

pub enum Ast {
    Expression(Expression),
    Statement(Statement)
}