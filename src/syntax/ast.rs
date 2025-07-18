use crate::syntax::lexer::Token;
use crate::syntax::traits::PartialTreeEq;

#[derive(Debug, Clone)]
pub struct Identifier {
    pub name: Token,
}

pub struct Variable {

}

#[derive(Debug, Clone)]
pub struct Literal {

}

#[derive(Debug)]
pub struct Type {
}

#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    pub name: Token,
    pub is_mut: bool,
}

impl PartialTreeEq for TypeAnnotation {
    type Other = TypeAnnotation;

    fn partial_eq(&self, other: &TypeAnnotation) -> bool {
        self.name.partial_eq(&other.name)
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
    Literal(Literal),
    Identifier(Identifier),
    MethodCall(),
    DotAccess(),
    Call(),
    ArraySlice(),
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
    pub return_type: Option<Token>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct Method {
    pub name: Token,
    pub bound_type: Token,
    pub is_mut_self: bool,
    pub arguments: Vec<TypedDeclaration>,
    pub return_type: Option<Token>,
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
            ) => ta.partial_eq(tb) && na.partial_eq(nb) &&
                fa.iter()
                    .zip(fb)
                    .all(|(a, b)| a.partial_eq(b)),
            _ => false
        }
    }
}

pub enum Ast {
    Expression(Expression),
    Statement(Statement)
}