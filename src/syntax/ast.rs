use std::fmt::Pointer;

use crate::syntax::lexer::Token;

static mut AST_ID_COUNTER: usize = 0;

pub fn next_id() -> usize {
    let id = unsafe {
        AST_ID_COUNTER
    };
    
    unsafe {
        AST_ID_COUNTER += 1;
    }

    id
}

pub fn current_id() -> usize {
    unsafe {
        AST_ID_COUNTER
    }
}

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

#[derive(Debug, Clone)]
pub struct Grouping {
    pub token: Token,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct DotAccess {
    pub object: Box<Expression>,
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct ArrowAccess {
    pub pointer: Box<Expression>,
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub token: Token,
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct ArraySlice {
    pub token: Token,
    pub array_expression: Box<Expression>,
    pub slice_expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub token: Token,
    pub operator: Token,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Cast {
    pub token: Token,
    pub left: Box<Expression>,
    pub target_type: TypeAnnotation,
    pub is_reinterpret_cast: bool, 
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Range {
    pub token: Token,
    pub start: Box<Expression>,
    pub end: Box<Expression>,
    pub inclusive: bool,
}

#[derive(Debug, Clone)]
pub struct InplaceAssignment {
    pub token: Token,
    pub lhs: Box<Expression>,
    pub operator: Token,
    pub rhs: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub token: Token,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct IfElseExpression {
    pub token: Token,
    pub condition: Box<Expression>,
    pub then_branch: BlockExpression,
    pub else_branch: BlockExpression,
}

#[derive(Debug, Clone)]
pub struct BlockExpression {
    pub token: Token,
    pub statements: Vec<Statement>,
    pub return_expression: Option<Box<Expression>>
}

#[derive(Debug, Clone)]
pub struct SelfExpression {
    pub token: Token,
}

#[derive(Debug, Clone)]
pub struct FnExpression {
    pub token: Token,
    pub function: Function,
}

#[derive(Debug, Clone)]
pub struct StructInitializer {
    pub token: Token,
    pub struct_name: Token,
    pub field_initializers: Vec<(Token, Expression)>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Grouping(Grouping),
    Literal(Literal),
    Identifier(Identifier),
    MethodCall(),
    DotSet {},
    ArrowSet {},
    DotAccess(DotAccess),
    ArrowAccess(ArrowAccess),
    Call(Call),
    ArraySlice(ArraySlice),
    Unary(Unary),
    Cast(Cast),
    Binary(Binary),
    // LogicalBinary {
    //     token: Token,
    //     left: Box<Expression>,
    //     operator: Token,
    //     right: Box<Expression>,
    // },
    Range(Range),
    InplaceAssignment(InplaceAssignment),
    Assignment(Assignment),
    IfElseExpression(IfElseExpression),
    Block(BlockExpression),
    SelfExpression(SelfExpression),
    FnExpression(FnExpression),
    StructInitializer(StructInitializer),
}

impl Expression {
    pub fn new_block(
        token: Token,
        statements: Vec<Statement>,
        return_expression: Option<Box<Expression>>,
    ) ->  Expression {
        Expression::Block(BlockExpression {
            token, statements, return_expression 
        })
    }
    
    pub fn new_grouping(token: Token, expression: Expression) -> Expression {
        Expression::Grouping(Grouping {
            token, expression: Box::new(expression)
        })
    }
    
    pub fn new_identifier(name: Token) -> Expression {
        Expression::Identifier(Identifier { name })
    }
    
    pub fn new_self(token: Token) -> Expression {
        Expression::SelfExpression(SelfExpression { token })
    }
    
    pub fn new_fn(token: Token, function: Function) -> Expression {
        Expression::FnExpression(FnExpression { token, function })
    }
    
    pub fn new_struct_initializer(
        token: Token,
        struct_name: Token,
        field_initializers: Vec<(Token, Expression)>,
    ) -> Expression {
        Expression::StructInitializer(StructInitializer { 
            token, struct_name, field_initializers
        })
    }
    
    pub fn new_cast(
        token: Token,
        left: Expression,
        target_type: TypeAnnotation,
        is_reinterpret_cast: bool,
    ) -> Expression {
        Expression::Cast(Cast {
            token,
            left: Box::new(left),
            target_type,
            is_reinterpret_cast,
        })
    }
    
    pub fn new_dot_access(
        object: Expression,
        name: Token,
    ) -> Expression {
        Expression::DotAccess(DotAccess {
            object: Box::new(object),
            name,
        })
    }

    pub fn new_arrow_access(
        pointer: Expression,
        name: Token,
    ) -> Expression {
        Expression::ArrowAccess(ArrowAccess {
            pointer: Box::new(pointer),
            name,
        })
    }
    
    pub fn new_call(
        token: Token,
        callee: Expression,
        arguments: Vec<Expression>,
    ) -> Expression {
        Expression::Call(Call {
            token,
            callee: Box::new(callee),
            arguments,
        })
    }
   
    pub fn new_array_slice(
        token: Token,
        array_expression: Expression,
        slice_expression: Expression,
    ) -> Expression {
        Expression::ArraySlice(ArraySlice {
            token,
            array_expression: Box::new(array_expression),
            slice_expression: Box::new(slice_expression),
        })
    }
    
    pub fn new_unary(token: Token, operator: Token, right: Expression) -> Expression {
        Expression::Unary(Unary {
            token,
            operator,
            expression: Box::new(right),
        })
    }
    
    pub fn new_binary(
        left: Expression,
        operator: Token,
        right: Expression,
    ) -> Expression {
        Expression::Binary(Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }
    
    pub fn new_assignment(
        token: Token,
        left: Expression,
        right: Expression,
    ) -> Expression {
        Expression::Assignment(Assignment {
            token,
            lhs: Box::new(left),
            rhs: Box::new(right),
        })
    }
    
    pub fn new_inplace_assignment(
        token: Token,
        left: Expression,
        operator: Token,
        right: Expression,
    ) -> Expression {
        Expression::InplaceAssignment(InplaceAssignment {
            token,
            lhs: Box::new(left),
            operator,
            rhs: Box::new(right),
        })
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Function {
    pub name: Token,
    pub arguments: Vec<TypedDeclaration>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Method {
    pub name: Token,
    pub bound_type: Token,
    pub is_mut_self: bool,
    pub arguments: Vec<TypedDeclaration>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum ImplFunction {
    Function(Function),
    Method(Method),
}

pub struct EnumVariant {}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub token: Token,
    pub name: Token,
    pub variable_type: Option<TypeAnnotation>,
    pub initializer: Option<Box<Expression>>,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct StaticStatement { //+-
    pub token: Token,
    pub name: Token,
    pub variable_type: TypeAnnotation,
    pub initializer: Box<Expression>,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct ConstStatement{ //+-
    pub token: Token,
    pub name: Token,
    pub variable_type: TypeAnnotation,
    pub initializer: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct WhileStatement { //++
    pub token: Token,
    pub condition: Box<Expression>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct BreakStatement {
    pub token: Token,
    pub loop_key: Option<Token>
}

#[derive(Debug, Clone)]
pub struct ContinueStatement {
    pub token: Token,
    pub loop_key: Option<Token>
}

#[derive(Debug, Clone)]
pub struct FnStatement{ //+
    pub token: Token,
    pub function: ImplFunction,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement { //+
    pub token: Token,
    pub expression: Option<Box<Expression>>
}

#[derive(Debug, Clone)]
pub struct DeferStatement {
    pub token: Token,
    pub call_expression: Box<Expression>,
    pub to_closest_block: bool
}

#[derive(Debug, Clone)]
pub struct StructStatement {
    pub token: Token,
    pub name: Token,
    pub fields: Vec<TypedDeclaration>,
}

#[derive(Debug, Clone)]
pub struct ImplStatement {
    pub token: Token,
    pub implemented_type: Token,
    pub top_level_statements: Vec<Statement>,
    pub functions: Vec<ImplFunction>,
}

#[derive(Debug, Clone)]
pub struct IfElseStatement {
    pub token: Token,
    pub condition: Box<Expression>,
    pub then_branch: Vec<Statement>,
    pub else_branch: Option<Vec<Statement>>,
}


#[derive(Debug, Clone)]
pub enum Statement {
    EmptyStatement { //++
        semicolon_token: Token,
    },
    LetStatement(LetStatement),
    StaticStatement(StaticStatement),
    ConstStatement(ConstStatement),
    ExpressionStatement(ExpressionStatement),
    WhileStatement(WhileStatement),
    BreakStatement(BreakStatement),
    ContinueStatement(ContinueStatement),
    FnStatement(FnStatement),
    ReturnStatement(ReturnStatement),
    DeferStatement(DeferStatement),
    StructStatement(StructStatement),
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

    ImplStatement(ImplStatement),
    IfElseStatement(IfElseStatement),
}

impl Statement {
    pub fn new_defer(token: Token, call: Expression, to_closest_block: bool) -> Self {
        let defer = DeferStatement {
            token,
            call_expression: Box::new(call),
            to_closest_block,
        };
        Statement::DeferStatement(defer)
    }
    
    pub fn new_let(
        token: Token,
        name: Token,
        variable_type: Option<TypeAnnotation>,
        initializer: Option<Box<Expression>>,
        is_mut: bool,
    ) -> Self {
        Statement::LetStatement(LetStatement {
            token,
            name,
            variable_type,
            initializer,
            is_mut,
        })
    }

    pub fn new_static(
        token: Token,
        name: Token,
        variable_type: TypeAnnotation,
        initializer: Box<Expression>,
        is_mut: bool,
    ) -> Self {
        Statement::StaticStatement(StaticStatement {
            token,
            name,
            variable_type,
            initializer,
            is_mut,
        })
    }

    pub fn new_const(
        token: Token,
        name: Token,
        variable_type: TypeAnnotation,
        initializer: Box<Expression>,
    ) -> Self {
        Statement::ConstStatement(ConstStatement {
            token,
            name,
            variable_type,
            initializer,
        })
    }
    
    pub fn new_fn(token: Token, function: ImplFunction) -> Self {
        Statement::FnStatement(FnStatement{
            token,
            function,
        })
    }
    
    pub fn new_struct(
        token: Token,
        name: Token,
        fields: Vec<TypedDeclaration>,
    ) -> Self {
        Statement::StructStatement(StructStatement {
            token,
            name,
            fields,
        })
    }
    
    pub fn new_impl(
        token: Token,
        implemented_type: Token,
        top_level_statements: Vec<Statement>,
        functions: Vec<ImplFunction>,
    ) -> Self {
        
        
        Statement::ImplStatement(ImplStatement {
            token,
            implemented_type,
            top_level_statements,
            functions,
        })
    }
    
    pub fn new_expression(expression: Expression) -> Self {
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(expression),
        })
    }
    
    pub fn new_if_else(
        token: Token,
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    ) -> Self {
        Statement::IfElseStatement(IfElseStatement {
            token,
            condition: Box::new(condition),
            then_branch,
            else_branch,
        })
    }
    
    pub fn new_while(
        token: Token,
        condition: Expression,
        body: Vec<Statement>,
    ) -> Self {
        Statement::WhileStatement(WhileStatement {
            token,
            condition: Box::new(condition),
            body,
        })
    }
    
    pub fn new_break(token: Token, loop_key: Option<Token>) -> Self {
        Statement::BreakStatement(BreakStatement {
            token,
            loop_key,
        })
    }

    pub fn new_continue(token: Token, loop_key: Option<Token>) -> Self {
        Statement::ContinueStatement(ContinueStatement {
            token,
            loop_key,
        })
    }
    
    pub fn new_return(token: Token, expression: Option<Box<Expression>>) -> Self {
        Statement::ReturnStatement(ReturnStatement {
            token,
            expression,
        })
    }
}

pub enum Ast {
    Expression(Expression),
    Statement(Statement)
}