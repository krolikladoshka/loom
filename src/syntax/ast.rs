use std::fmt::Pointer;

use crate::syntax::lexer::Token;

static mut AST_ID_COUNTER: usize = 0;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AstNodeIndex(pub usize);


impl AstNodeIndex {
    pub fn increment(&mut self) -> AstNodeIndex {
        let result = AstNodeIndex(self.0);
        self.0 += 1;

        result
    }
}

impl Into<usize> for AstNodeIndex {
    fn into(self) -> usize {
        self.0
    }
}


pub fn next_id() -> AstNodeIndex {
    let id = unsafe {
        AST_ID_COUNTER
    };

    unsafe {
        AST_ID_COUNTER += 1;
    }

    AstNodeIndex(id)
}

pub fn current_id() -> AstNodeIndex {
    unsafe {
        AstNodeIndex(AST_ID_COUNTER)
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub node_id: AstNodeIndex,
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
        // node_id: AstNodeIndex,
        token: Token,
        value: u8,
    },
    U16 {
        // node_id: AstNodeIndex,
        token: Token,
        value: u16,
    },
    U32 {
        // node_id: AstNodeIndex,
        token: Token,
        value: u32,
    },
    U64 {
        // node_id: AstNodeIndex,
        token: Token,
        value: u64,
    },
    I8 {
        // node_id: AstNodeIndex,
        token: Token,
        value: i8,
    },
    I16 {
        // node_id: AstNodeIndex,
        token: Token,
        value: i16,
    },
    I32 {
        // node_id: AstNodeIndex,
        token: Token,
        value: i32,
    },
    I64 {
        // node_id: AstNodeIndex,
        token: Token,
        value: i64,
    },
    F32 {
        // node_id: AstNodeIndex,
        token: Token,
        value: f32,
    },
    F64 {
        // node_id: AstNodeIndex,
        token: Token,
        value: f64,
    },
    Bool {
        // node_id: AstNodeIndex,
        token: Token,
        value: bool,
    },
    Char {
        // node_id: AstNodeIndex,
        token: Token,
        value: char,
    },
    String {
        // node_id: AstNodeIndex,
        token: Token,
        value: String,
    },
    MultilineString {
        // node_id: AstNodeIndex,
        token: Token,
        value: String,
    },
}

impl Literal {
    // pub fn get_node_id(&self) -> AstNodeIndex {
    //     match self {
    //         Literal::U8 { node_id, .. } => *node_id,
    //         Literal::U16 { node_id, .. } => *node_id,
    //         Literal::U32 { node_id, .. } => *node_id,
    //         Literal::U64 { node_id, .. } => *node_id,
    //         Literal::I8 { node_id, .. } => *node_id,
    //         Literal::I16 { node_id, .. } => *node_id,
    //         Literal::I32 { node_id, .. } => *node_id,
    //         Literal::I64 { node_id, .. } => *node_id,
    //         Literal::F32 { node_id, .. } => *node_id,
    //         Literal::F64 { node_id, .. } => *node_id,
    //         Literal::Bool { node_id, .. } => *node_id,
    //         Literal::Char { node_id, .. } => *node_id,
    //         Literal::String { node_id, .. } => *node_id,
    //         Literal::MultilineString { node_id, .. } => *node_id,
    //     }
    // }

    pub fn new_u8(token: Token, value: u8) -> Self {
        Self::U8 {
            // node_id: next_id(),
            token,
            value,
        }
    }

    pub fn new_u16(token: Token, value: u16) -> Self {
        Self::U16 {
            // node_id: next_id(),
            token,
            value,
        }
    }

    pub fn new_u32(token: Token, value: u32) -> Self {
        Self::U32 {
            // node_id: next_id(),
            token,
            value,
        }
    }

    pub fn new_u64(token: Token, value: u64) -> Self {
        Self::U64 {
            // node_id: next_id(),
            token,
            value,
        }
    }

    pub fn new_i8(token: Token, value: i8) -> Self {
        Self::I8 {
            // node_id: next_id(),
            token,
            value,
        }
    }

    pub fn new_i16(token: Token, value: i16) -> Self {
        Self::I16 {
            // node_id: next_id(),
            token,
            value,
        }
    }

    pub fn new_i32(token: Token, value: i32) -> Self {
        Self::I32 {
            // node_id: next_id(),
            token,
            value,
        }
    }

    pub fn new_i64(token: Token, value: i64) -> Self {
        Self::I64 {
            // node_id: next_id(),
            token,
            value,
        }
    }

    pub fn new_bool(token: Token, value: bool) -> Self {
        Self::Bool {
            // node_id: next_id(),
            token,
            value,
        }
    }

    pub fn new_char(token: Token, value: char) -> Self {
        Self::Char {
            // node_id: next_id(),
            token,
            value,
        }
    }

    pub fn new_string(token: Token, value: String) -> Self {
        Self::String {
            // node_id: next_id(),
            token,
            value,
        }
    }

    pub fn new_multiline_string(token: Token, value: String) -> Self {
        Self::MultilineString {
            // node_id: next_id(),
            token,
            value,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LiteralNode {
    pub node_id: AstNodeIndex,
    pub literal: Literal,
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
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct DotAccess {
    pub node_id: AstNodeIndex,
    pub object: Box<Expression>,
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct ArrowAccess {
    pub node_id: AstNodeIndex,
    pub pointer: Box<Expression>,
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct ArraySlice {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub array_expression: Box<Expression>,
    pub slice_expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub operator: Token,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Cast {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub left: Box<Expression>,
    pub target_type: TypeAnnotation,
    pub is_reinterpret_cast: bool, 
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub node_id: AstNodeIndex,
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Range {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub start: Box<Expression>,
    pub end: Box<Expression>,
    pub inclusive: bool,
}

#[derive(Debug, Clone)]
pub struct InplaceAssignment {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub lhs: Box<Expression>,
    pub operator: Token,
    pub rhs: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct IfElseExpression {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub condition: Box<Expression>,
    pub then_branch: BlockExpression,
    pub else_branch: BlockExpression,
}

#[derive(Debug, Clone)]
pub struct BlockExpression {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub statements: Vec<Statement>,
    pub return_expression: Option<Box<Expression>>
}

#[derive(Debug, Clone)]
pub struct SelfExpression {
    pub node_id: AstNodeIndex,
    pub token: Token,
}

#[derive(Debug, Clone)]
pub struct FnExpression {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub function: Function,
}

#[derive(Debug, Clone)]
pub struct StructInitializer {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub struct_name: Token,
    pub field_initializers: Vec<(Token, Expression)>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Grouping(Grouping),
    Literal(LiteralNode),
    Identifier(Identifier),
    MethodCall {node_id: AstNodeIndex},
    DotSet {node_id: AstNodeIndex},
    ArrowSet {node_id: AstNodeIndex},
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
    pub fn new_literal(literal: Literal) -> Expression {
        Self::Literal(LiteralNode {
            node_id: next_id(),
            literal,
        })
    }
    
    pub fn new_block(
        token: Token,
        statements: Vec<Statement>,
        return_expression: Option<Box<Expression>>,
    ) ->  Expression {
        Expression::Block(BlockExpression {
            node_id: next_id(),
            token, statements, return_expression 
        })
    }
    
    pub fn new_grouping(token: Token, expression: Expression) -> Expression {
        Expression::Grouping(Grouping {
            node_id: next_id(),
            token, expression: Box::new(expression)
        })
    }
    
    pub fn new_identifier(name: Token) -> Expression {
        Expression::Identifier(Identifier { node_id: next_id(),  name })
    }
    
    pub fn new_self(token: Token) -> Expression {
        Expression::SelfExpression(SelfExpression { node_id: next_id(), token })
    }
    
    pub fn new_fn(token: Token, function: Function) -> Expression {
        Expression::FnExpression(FnExpression { node_id: next_id(), token, function })
    }
    
    pub fn new_struct_initializer(
        token: Token,
        struct_name: Token,
        field_initializers: Vec<(Token, Expression)>,
    ) -> Expression {
        Expression::StructInitializer(StructInitializer {
            node_id: next_id(),
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
            node_id: next_id(),
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
            node_id: next_id(),
            object: Box::new(object),
            name,
        })
    }

    pub fn new_arrow_access(
        pointer: Expression,
        name: Token,
    ) -> Expression {
        Expression::ArrowAccess(ArrowAccess {
            node_id: next_id(),
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
            node_id: next_id(),
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
            node_id: next_id(),
            token,
            array_expression: Box::new(array_expression),
            slice_expression: Box::new(slice_expression),
        })
    }
    
    pub fn new_unary(token: Token, operator: Token, right: Expression) -> Expression {
        Expression::Unary(Unary {
            node_id: next_id(),
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
            node_id: next_id(),
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
            node_id: next_id(),
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
            node_id: next_id(),
            token,
            lhs: Box::new(left),
            operator,
            rhs: Box::new(right),
        })
    }

    pub fn get_node_id(&self) -> AstNodeIndex {
        match self {
            Expression::Grouping(x) => x.node_id,
            Expression::Literal(x) => x.node_id,
            Expression::Identifier(x) => x.node_id,
            Expression::MethodCall { node_id } => *node_id,
            Expression::DotSet { node_id } => *node_id,
            Expression::ArrowSet { node_id } => *node_id,
            Expression::DotAccess(x) => x.node_id,
            Expression::ArrowAccess(x) => x.node_id,
            Expression::Call(x) => x.node_id,
            Expression::ArraySlice(x) => x.node_id,
            Expression::Unary(x) => x.node_id,
            Expression::Cast(x) => x.node_id,
            Expression::Binary(x) => x.node_id,
            Expression::Range(x) => x.node_id,
            Expression::InplaceAssignment(x) => x.node_id,
            Expression::Assignment(x) => x.node_id,
            Expression::IfElseExpression(x) => x.node_id,
            Expression::Block(x) => x.node_id,
            Expression::SelfExpression(x) => x.node_id,
            Expression::FnExpression(x) => x.node_id,
            Expression::StructInitializer(x) => x.node_id,
        }
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
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub name: Token,
    pub variable_type: Option<TypeAnnotation>,
    pub initializer: Option<Box<Expression>>,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct StaticStatement { //+-
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub name: Token,
    pub variable_type: TypeAnnotation,
    pub initializer: Box<Expression>,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct ConstStatement{ //+-
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub name: Token,
    pub variable_type: TypeAnnotation,
    pub initializer: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub node_id: AstNodeIndex,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct WhileStatement { //++
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub condition: Box<Expression>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct BreakStatement {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub loop_key: Option<Token>
}

#[derive(Debug, Clone)]
pub struct ContinueStatement {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub loop_key: Option<Token>
}

#[derive(Debug, Clone)]
pub struct FnStatement{ //+
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub function: ImplFunction,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement { //+
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub expression: Option<Box<Expression>>
}

#[derive(Debug, Clone)]
pub struct DeferStatement {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub call_expression: Box<Expression>,
    pub to_closest_block: bool
}

#[derive(Debug, Clone)]
pub struct StructStatement {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub name: Token,
    pub fields: Vec<TypedDeclaration>,
}

#[derive(Debug, Clone)]
pub struct ImplStatement {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub implemented_type: Token,
    pub top_level_statements: Vec<Statement>,
    pub functions: Vec<ImplFunction>,
}

#[derive(Debug, Clone)]
pub struct IfElseStatement {
    pub node_id: AstNodeIndex,
    pub token: Token,
    pub condition: Box<Expression>,
    pub then_branch: Vec<Statement>,
    pub else_branch: Option<Vec<Statement>>,
}


#[derive(Debug, Clone)]
pub enum Statement {
    EmptyStatement { //++
        node_id: AstNodeIndex,
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
    pub fn new_empty(token: Token) -> Self {
        Statement::EmptyStatement {
            node_id: next_id(),
            semicolon_token: token,
        }
    }

    pub fn new_defer(token: Token, call: Expression, to_closest_block: bool) -> Self {
        let defer = DeferStatement {
            node_id: next_id(),
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
            node_id: next_id(),
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
            node_id: next_id(),
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
            node_id: next_id(),
            token,
            name,
            variable_type,
            initializer,
        })
    }
    
    pub fn new_fn(token: Token, function: ImplFunction) -> Self {
        Statement::FnStatement(FnStatement{
            node_id: next_id(),
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
            node_id: next_id(),
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
            node_id: next_id(),
            token,
            implemented_type,
            top_level_statements,
            functions,
        })
    }
    
    pub fn new_expression(expression: Expression) -> Self {
        Statement::ExpressionStatement(ExpressionStatement {
            node_id: next_id(),
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
            node_id: next_id(),
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
            node_id: next_id(),
            token,
            condition: Box::new(condition),
            body,
        })
    }
    
    pub fn new_break(token: Token, loop_key: Option<Token>) -> Self {
        Statement::BreakStatement(BreakStatement {
            node_id: next_id(),
            token,
            loop_key,
        })
    }

    pub fn new_continue(token: Token, loop_key: Option<Token>) -> Self {
        Statement::ContinueStatement(ContinueStatement {
            node_id: next_id(),
            token,
            loop_key,
        })
    }
    
    pub fn new_return(token: Token, expression: Option<Box<Expression>>) -> Self {
        Statement::ReturnStatement(ReturnStatement {
            node_id: next_id(),
            token,
            expression,
        })
    }

    pub fn get_node_id(&self) -> AstNodeIndex {
        match self {
            Statement::EmptyStatement { node_id, ..} => *node_id,
            Statement::LetStatement(x) => x.node_id,
            Statement::StaticStatement(x) => x.node_id,
            Statement::ConstStatement(x) => x.node_id,
            Statement::ExpressionStatement(x) => x.node_id,
            Statement::WhileStatement(x) => x.node_id,
            Statement::BreakStatement(x) => x.node_id,
            Statement::ContinueStatement(x) => x.node_id,
            Statement::FnStatement(x) => x.node_id,
            Statement::ReturnStatement(x) => x.node_id,
            Statement::DeferStatement(x) => x.node_id,
            Statement::StructStatement(x) => x.node_id,
            Statement::ImplStatement(x) => x.node_id,
            Statement::IfElseStatement(x) => x.node_id,
        }
    }
}

pub enum Ast {
    Expression(Expression),
    Statement(Statement)
}

impl Ast {
    pub fn get_node_id(&self) -> AstNodeIndex {
        match self {
            Self::Expression(x) => x.get_node_id(),
            Self::Statement(x) => x.get_node_id(),
        }
    }
}