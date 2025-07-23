use std::fmt::{Display, Formatter};
use std::ops::Deref;
use crate::syntax::ast::{Binary, Cast, Expression, ExpressionStatement, FnStatement, Function, Grouping, Identifier, ImplFunction, ImplStatement, Literal, LiteralNode, Method, PointerAnnotation, ReturnStatement, Statement, StructInitializer, StructStatement, Type, TypeAnnotation, TypeKind, TypedDeclaration};
// use crate::syntax::ast::Expression::{Binary, Cast, Grouping, Identifier, StructInitializer};
// use crate::syntax::ast::Literal::{Bool, Char, MultilineString, F32, F64, I16, I32, I64, I8, U16, U32, U64, U8};
// use crate::syntax::ast::Statement::{EmptyStatement, ExpressionStatement, FnStatement, ImplStatement, ReturnStatement, StructStatement};
use crate::syntax::lexer::Token;

pub trait PartialTreeEq {
    type Other: PartialTreeEq;
    fn partial_eq(&self, other: &Self::Other) -> bool;
}

pub trait TreePrint {
    fn print_tree(&self, indent: usize) -> String;
}


pub fn partial_eq<T: PartialTreeEq<Other = T>>(
    a: &T, b: &T
) -> bool {
    a.partial_eq(b)
}

pub fn partial_eq_all<T: PartialTreeEq<Other = T>>(
    a: &Vec<T>, b: &Vec<T>
) -> bool {
    if  a.len() != b.len() {
        return false;
    }
    
    let ab = a.iter()
        .zip(b.iter());

    for (a, b) in ab {
        if !a.partial_eq(b) {
            return false;
        }
    }

    true
}

impl PartialTreeEq for Literal {
    type Other = Literal;

    fn partial_eq(&self, other: &Self::Other) -> bool {
        use crate::syntax::ast::Literal::*;

        // TODO: well
        match (self, other) {
            (
                I8 { token: at, value: av },
                I8 { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                I16 { token: at, value: av },
                I16 { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                I32 { token: at, value: av },
                I32 { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                I64 { token: at, value: av },
                I64 { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                U8 { token: at, value: av },
                U8 { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                U16 { token: at, value: av },
                U16 { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                U32 { token: at, value: av },
                U32 { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                U64 { token: at, value: av },
                U64 { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                F32 { token: at, value: av },
                F32 { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                F64 { token: at, value: av },
                F64 { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                Bool { token: at, value: av },
                Bool { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                Char { token: at, value: av },
                Char { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                crate::syntax::ast::Literal::String { token: at, value: av },
                crate::syntax::ast::Literal::String { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            (
                MultilineString { token: at, value: av },
                MultilineString { token: bt, value: bv }
            ) => at.partial_eq(bt) && av == bv,
            _ => false,
        }
    }
}

impl PartialTreeEq for LiteralNode {
    type Other = LiteralNode;

    fn partial_eq(&self, other: &Self::Other) -> bool {
        self.literal.partial_eq(&other.literal)
    }
}

impl PartialTreeEq for Type {
    type Other = Type;

    fn partial_eq(&self, other: &Self::Other) -> bool {
        self.name.partial_eq(&other.name)
    }
}

impl PartialTreeEq for PointerAnnotation {
    type Other = PointerAnnotation;

    fn partial_eq(&self, other: &Self::Other) -> bool {
        self.points_to_mut == other.points_to_mut &&
            self.inner_type.partial_eq(&other.inner_type)
    }
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

impl PartialTreeEq for TypeAnnotation {
    type Other = TypeAnnotation;

    fn partial_eq(&self, other: &TypeAnnotation) -> bool {
        self.is_mut == other.is_mut &&
            self.kind.partial_eq(&other.kind)
        // self.name.partial_eq(&other.name)
    }
}

impl PartialTreeEq for TypedDeclaration {
    type Other = TypedDeclaration;

    fn partial_eq(&self, other: &TypedDeclaration) -> bool {
        self.name.partial_eq(&other.name) &&
            self.declared_type.partial_eq(&other.declared_type)
    }
}


impl PartialTreeEq for (Token, Expression) {
    type Other = (Token, Expression);

    fn partial_eq(&self, other: &Self::Other) -> bool {
        self.0.partial_eq(&other.0) &&
            self.1.partial_eq(&other.1)
    }
}

impl PartialTreeEq for Grouping {
    type Other = Grouping;
    fn partial_eq(&self, other: &Self) -> bool {
        self.token.partial_eq(&other.token) &&
            self.expression.partial_eq(&other.expression)
    }
}

impl PartialTreeEq for Identifier {
    type Other = Identifier;
    fn partial_eq(&self, other: &Self) -> bool {
        self.name.partial_eq(&other.name)
    }
}

impl PartialTreeEq for Binary {
    type Other = Binary;
    fn partial_eq(&self, other: &Self) -> bool {
        self.operator.partial_eq(&other.operator) &&
            self.left.partial_eq(&other.left) &&
            self.right.partial_eq(&other.right)
    }
}

impl PartialTreeEq for Vec<(Token, Expression)> {
    type Other = Vec<(Token, Expression)>;
    fn partial_eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .zip(other.iter())
            .all(|(a, b)| partial_eq(a, b))
    }
}

impl PartialTreeEq for StructInitializer {
    type Other = StructInitializer;

    fn partial_eq(&self, other: &Self) -> bool {
        self.token.partial_eq(&other.token) &&
            self.struct_name.partial_eq(&other.struct_name) &&
            self.field_initializers.partial_eq(&other.field_initializers)
    }
}

impl PartialTreeEq for Cast {
    type Other = Cast;
    fn partial_eq(&self, other: &Self) -> bool {
        self.is_reinterpret_cast == other.is_reinterpret_cast &&
            self.token.partial_eq(&other.token) &&
            self.target_type.partial_eq(&other.target_type) &&
            self.left.partial_eq(&other.left)
    }
}

impl PartialTreeEq for Expression {
    type Other = Expression;

    fn partial_eq(&self, other: &Self::Other) -> bool {
        use crate::syntax::ast::Expression::*;
        match (self, other) {
            (
                Literal(a),
                Literal(b),
            ) => partial_eq(a, b),
            (
                Grouping(a),
                Grouping(b)
            ) => partial_eq(a, b),
            (
                Identifier(a),
                Identifier(b),
            ) => partial_eq(a, b),
            (
                Binary(a),
                Binary(b)
            ) => partial_eq(a, b),
            (
                StructInitializer(a),
                StructInitializer(b)
            ) => partial_eq(a, b),
            (
                Cast(a),
                Cast(b)
            ) => partial_eq(a, b),
            _ => false,
        }
    }
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
            if !self.return_type.as_ref().unwrap()
                .partial_eq(&other.return_type.as_ref().unwrap()) {
                return false;
            }
        }

        partial_eq_all(&self.arguments, &other.arguments) &&
            partial_eq_all(&self.body, &other.body)
    }
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

impl PartialTreeEq for StructStatement {
    type Other = StructStatement;
    
    fn partial_eq(&self, other: &Self) -> bool {
        self.name.partial_eq(&other.name) &&
            self.token.partial_eq(&other.token) &&
            partial_eq_all(&self.fields, &other.fields)
    }
}

impl PartialTreeEq for FnStatement {
    type Other = FnStatement;
    
    fn partial_eq(&self, other: &Self) -> bool {
        self.token.partial_eq(&other.token) &&
            self.function.partial_eq(&other.function)
    }
}

impl PartialTreeEq for ImplStatement {
    type Other = ImplStatement;
    
    fn partial_eq(&self, other: &Self) -> bool {
        self.token.partial_eq(&other.token) &&
            self.implemented_type.partial_eq(&other.implemented_type) &&
            partial_eq_all(&self.top_level_statements, &other.top_level_statements) &&
            partial_eq_all(&self.functions, &other.functions)
    }
}

impl PartialTreeEq for ReturnStatement {
    type Other = ReturnStatement;
    
    fn partial_eq(&self, other: &Self) -> bool {
        if !self.token.partial_eq(&other.token) {
            return false;
        }
        
        if !((self.expression.is_none() && other.expression.is_none()) ||
            (self.expression.is_some() && other.expression.is_some()))
        {
            return false;
        }
        
        if self.expression.is_none() {
            return true;
        }
        
        partial_eq(
            self.expression.as_ref().unwrap().as_ref(),
            other.expression.as_ref().unwrap().as_ref()
        )
    }
}

impl PartialTreeEq for ExpressionStatement {
    type Other = ExpressionStatement;
    
    fn partial_eq(&self, other: &Self) -> bool {
        partial_eq(self.expression.as_ref(), other.expression.as_ref())
    }
}


impl PartialTreeEq for Statement {
    type Other = Statement;

    fn partial_eq(&self, other: &Statement) -> bool {
        use crate::syntax::ast::Statement::*;

        match (self, other) {
            (
                EmptyStatement { semicolon_token: a, .. },
                EmptyStatement { semicolon_token: b, ..}
            ) => a.partial_eq(b),
            (
                StructStatement(a),
                StructStatement(b)
            ) => partial_eq(a, b),
            (
                FnStatement(a),
                FnStatement(b)
            ) => partial_eq(a, b),
            (
                ImplStatement(a),
                ImplStatement(b)
            ) => partial_eq(a, b),
            (
                ReturnStatement(a),
                ReturnStatement(b)
            ) => partial_eq(a, b),
            (
                ExpressionStatement(a),
                ExpressionStatement(b)
            ) => partial_eq(a, b), 
            _ => false
        }
    }
}

//// tree print
/*
    todo: it doesn't work because of laziness, used only for hinting
    todo: still has to implement it for c transpiling
    example broken output
    fn fib(n:  ) ->   {
if n <= 1_u64 {
return n;} ;
    let prev = 0_u64;
    let current = 1_u64;
    let i = 0_u64;
    while i < n {
let temp_current = current;
    current = prev + current;
    prev = temp_current;
    ++i;
};
    return current;
};
    fn factorial(n:  ) ->   {
if n <= 1_u64 {
return 1_u64;} ;
    return n * factorial(0n - 1_u64);
};
struct LinkedListNode;
 */
// todo! do i even need it . . .
// fn prev_indent(indent: usize) -> usize {
//     if indent >= 4 {
//         indent - 4
//     } else {
//         0
//     }
// }
// 
// 
// impl Display for TypeKind {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             TypeKind::Simple(t) => write!(
//                 f, "{}", t.name.literal
//             ),
//             TypeKind::Pointer(t) => write!(
//                 f, "*{} {}",
//                 if t.points_to_mut { "mut" } else { "const" },
//                 t.inner_type
//             )
//         }
//     }
// }
// 
// impl Display for TypeAnnotation {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f, "{} {}",
//             if self.is_mut { "mut" } else { "" },
//             self.kind
//         )
//     }
// }
// 
// impl Display for TypedDeclaration {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f, "{}: {}",
//             self.name.literal,
//             self.declared_type
//         )
//     }
// }
// 
// impl TreePrint for Literal {
//     fn print_tree(&self, indent: usize) -> String {
//         use Literal::*;
//         let indent = " ".repeat(indent);
// 
//         match self {
//             U8 { value, .. } =>
//                 format!("{}{}_u8", indent, value),
//             U16 { value, .. } =>
//                 format!("{}{}_u16", indent, value),
//             U32 { value, .. } =>
//                 format!("{}{}_u32", indent, value),
//             U64 { value, .. } =>
//                 format!("{}{}_u64", indent, value),
//             I8 { value, .. } =>
//                 format!("{}{}_i8", indent, value),
//             I16 { value, .. } =>
//                 format!("{}{}_i16", indent, value),
//             I32 { value, .. } =>
//                 format!("{}{}_i32", indent, value),
//             I64 { value, .. } =>
//                 format!("{}{}_i64", indent, value),
//             F32 { value, .. } =>
//                 format!("{}{}_f32", indent, value),
//             F64 { value, .. } =>
//                 format!("{}{}_f64", indent, value),
//             Bool { value, .. } =>
//                 format!("{}{}_bool", indent, value),
//             Char { value, .. } =>
//                 format!("{}'{}'_char", indent, value),
//             String { value, .. } =>
//                 format!("{}\"{}\"_string", indent, value),
//             MultilineString {value, .. } =>
//                 format!("{}\"{}\"_multiline_string", indent, value),
//         }
//     }
// }
// 
// impl TreePrint for Vec<Expression> {
//     fn print_tree(&self, indent: usize) -> String {
//         let list = self.iter().map(|v| v.print_tree(0))
//             .collect::<Vec<_>>().join(", ");
// 
//         format!("{}{}", indent, list)
//     }
// }
// 
// 
// impl TreePrint for Grouping {
//     fn print_tree(&self, indent: usize) -> String {
//         format!("({})", self.expression.print_tree(indent))
//     }
// }
// 
// 
// impl TreePrint for Expression {
//     fn print_tree(&self, indent: usize) -> String {
//         use Expression::*;
// 
//         let ts = " ".repeat(indent);
//         let pi = prev_indent(indent);
// 
//         match self {
//             Grouping(g) => format!("{}{}", ts, g.print_tree(indent)), 
//             Literal(q) => format!("{}{}", ts, q.print_tree(pi)),
//             Identifier {
//                 name,
//             } => format!("{}{}", ts, name.literal),
//             DotAccess {
//                 object,
//                 name,
//             } => format!("{}{}.{}", ts, object.print_tree(pi), name.literal),
//             ArrowAccess {
//                 pointer,
//                 name,
//             } => format!("{}{}->{}", ts, pointer.print_tree(pi), name.literal),
//             Call {
//                 callee,
//                 arguments,
//                 ..
//             } => format!("{}{}({})", ts, callee.print_tree(0), arguments.print_tree(0)),
//             ArraySlice {
//                 array_expression,
//                 slice_expression,
//                 ..
//             } => format!(
//                 "{}{}[{}]", ts,
//                 array_expression.print_tree(0),
//                 slice_expression.print_tree(0),
//             ),
//             Unary {
//                 operator,
//                 expression,
//                 ..
//             } => format!(
//                 "{}{}{}",
//                 ts,
//                 operator.lexeme,
//                 expression.print_tree(0),
//             ),
//             Cast {
//                 left,
//                 target_type,
//                 is_reinterpret_cast,
//                 ..
//             } => format!(
//                 "{}{} {} {}",
//                 ts,
//                 left.print_tree(0),
//                 if *is_reinterpret_cast { "as raw" } else { "as" },
//                 target_type
//             ),
//             Binary {
//                 left,
//                 operator,
//                 right,
//             } => format!(
//                 "{}{} {} {}",
//                 ts,
//                 left.print_tree(0),
//                 operator.lexeme,
//                 right.print_tree(0)
//             ),
//             // Range {
//             //     token: Token,
//             //     start: Box<Expression>,
//             //     end: Box<Expression>,
//             //     inclusive: bool,
//             // },
//             InplaceAssignment {
//                 lhs,
//                 operator,
//                 rhs,
//                 ..
//             } => format!(
//                 "{}{} {} {}",
//                 ts,
//                 lhs.print_tree(0),
//                 operator.lexeme,
//                 rhs.print_tree(0)
//             ),
//             Assignment {
//                 lhs,
//                 rhs,
//                 ..
//             } => format!(
//                 "{}{} = {}",
//                 ts,
//                 lhs.print_tree(0),
//                 rhs.print_tree(0)
//             ),
//             // IfElseExpression {
//             //     token: Token,
//             //     condition: Box<Expression>,
//             //     then_branch: Box<Statement>,
//             //     else_branch: Box<Statement>,
//             // },
//             // Block {
//             //     token: Token,
//             //     statements: Vec<Statement>,
//             //     return_expression: Option<Box<Expression>>,
//             // },
//             SelfExpression { .. } => "self".to_string(),
//             // FnExpression {
//             //     token: Token,
//             //     function: Function,
//             // },
//             StructInitializer {
//                 struct_name,
//                 field_initializers,
//                 ..
//             } => format!(
//                 "{}{} initializer",
//                 ts,
//                 struct_name.literal,
//             ),
//             _ => todo!()
//         }
//     }
// }
// 
// impl<Printable> TreePrint for Option<Printable>
// where
//     Printable: Deref,
//     Printable::Target: TreePrint,
// {
//     fn print_tree(&self, indent: usize) -> String {
//         if let Some(printable) = self {
//             printable.print_tree(indent)
//         } else {
//             String::new()
//         }
//     }
// }
// 
// 
// impl TreePrint for Vec<Statement> {
//     fn print_tree(&self, indent: usize) -> String {
// 
//         let list = self.iter()
//             .map(
//                 |v| format!("{};", v.print_tree(0))
//             ).collect::<Vec<_>>().join(format!("\n{}", " ".repeat(indent)).as_str());
//         
//         list
//     }
// }
// 
// 
// impl TreePrint for Function {
//     fn print_tree(&self, indent: usize) -> String {
//         format!(
//             "{}fn {}({}) -> {} {{\n{}\n}}",
//             " ".repeat(indent),
//             self.name.literal,
//             self.arguments.iter()
//                 .map(|td| format!("{}", td))
//                 .collect::<Vec<_>>()
//                 .join(", "),
//             if let Some(rt) = self.return_type.as_ref() {
//                 rt.to_string()
//             } else {
//                 "void".to_string()
//             },
//             self.body.print_tree(indent)
//         )
//     }
// }
// 
// impl TreePrint for Method {
//     fn print_tree(&self, indent: usize) -> String {
//         format!(
//             "{}fn {}({}, {}) -> {} {{\n{}\n}}",
//             " ".repeat(indent),
//             self.name.literal,
//             if self.is_mut_self {
//                 "*mut self"
//             } else {
//                 "*self"
//             },
//             self.arguments.iter()
//                 .map(|td| format!("{}", td))
//                 .collect::<Vec<_>>()
//                 .join(", "),
//             if let Some(rt) = self.return_type.as_ref() {
//                 rt.to_string()
//             } else {
//                 "void".to_string()
//             },
//             self.body.print_tree(indent)
//         )
//     }
// }
// 
// impl TreePrint for ImplFunction {
//     fn print_tree(&self, indent: usize) -> String {
//         use ImplFunction::*;
//         
//         match self {
//             Function(f) => f.print_tree(indent),
//             Method(m) => m.print_tree(indent),
//         }
//     }
// }
// 
// impl TreePrint for Vec<ImplFunction> {
//     fn print_tree(&self, indent: usize) -> String {
//         self.iter()
//             .map(|f| f.print_tree(indent))
//             .collect::<Vec<_>>()
//             .join("\n")
//     }
// }
// 
// impl TreePrint for Statement {
//     fn print_tree(&self, indent: usize) -> String {
//         use Statement::*;
//         let ts = " ".repeat(indent);
// 
//         match self {
//             EmptyStatement { .. } => format!("{};", indent),
//             LetStatement { //+-
//                 name,
//                 variable_type,
//                 initializer,
//                 is_mut,
//                 ..
//             } => format!(
//                 "{}let {} = {}", ts, name.literal, initializer.print_tree(0)
//             ),
//             StaticStatement { //+-
//                 // name,
//                 // variable_type,
//                 // initializer,
//                 // is_mut,
//                 ..
//             } => format!("{}static statement", ts),
//             ConstStatement { //+-
//                 // token: Token,
//                 // name: Token,
//                 // variable_type: TypeAnnotation,
//                 // initializer: Box<Expression>,
//                 ..
//             } => format!("{}const statement", ts),
//             ExpressionStatement { //++
//                 expression
//             } => format!("{}{}", ts, expression.print_tree(0)),
//             WhileStatement { //++
//                 condition, 
//                 body,
//                 ..
//             } => format!(
//                 "{}while {} {{\n{}\n}}",
//                 ts,
//                 condition.print_tree(0),
//                 body.print_tree(indent + 4)
//             ),
//             BreakStatement {
//                 ..
//             } => format!("{}break", ts),
//             ContinueStatement { //+
//                 ..
//             } => format!("{}continue", ts),
//             FnStatement { //+
//                 function,
//                 ..
//             } => format!("{}{}", ts, function.print_tree(indent + 4)),
//             ReturnStatement { //+
//                 expression,
//                 ..
//             } => format!("{}return {}", ts, expression.print_tree(0)),
//             DeferStatement {
//                 ..
//             } => format!("{}defer", ts),
//             StructStatement {
//                 name,
//                 ..
//             } => format!("{}struct {}", ts, name.literal),
//             // UnionStructStatement {
//             //     token: Token,
//             //     name: Token,
//             //     fields: Vec<SizedTypedDeclaration>
//             // },
//             // EnumStructStatement {
//             //     token: Token,
//             //     name: Token,
//             //     variants: Vec<EnumVariant>
//             // },
// 
//             ImplStatement {
//                 implemented_type,
//                 functions,
//                 ..
//             } => format!(
//                 "{}impl {} {{\n{}\n}}",
//                 ts, implemented_type.literal,
//                 functions.print_tree(indent + 4),
//             ),
//             IfElseStatement {
//                 condition,
//                 then_branch,
//                 else_branch,
//                 ..
//             } => {
//                 let mut result = vec![];
//                 
//                 result.push(
//                     format!(
//                         "{}if {} {{\n{}}} ",
//                         ts,
//                         condition.print_tree(0),
//                         then_branch.print_tree(indent + 4)
//                     )
//                 );
//                 
//                 if let  Some(else_branch) = else_branch {
//                     result.push(
//                         format!(
//                             "else {{\n{}\n}}",
//                             else_branch.print_tree(indent + 4)
//                         )
//                     );
//                 }
//                 
//                 result.join(" ")
//             }
//         }
//     }
// }