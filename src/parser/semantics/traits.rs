// use crate::parser::parser::Parser;
// use crate::syntax::ast::{Expression, Statement};
// 
// pub trait TreeSemantics
// where
// {
//     type Error;
// 
//     fn analyze_expression<SemanticsResult>(&mut self, expression: Expression)
//         -> Result<SemanticsResult, Self::Error>
//     {
//         match expression {
//             Expression::Grouping { .. } => {}
//             Expression::Literal(_) => {}
//             Expression::Identifier { .. } => {}
//             Expression::MethodCall() => {}
//             Expression::DotSet { .. } => {}
//             Expression::ArrowSet { .. } => {}
//             Expression::DotAccess { .. } => {}
//             Expression::ArrowAccess { .. } => {}
//             Expression::Call { .. } => {}
//             Expression::ArraySlice { .. } => {}
//             Expression::Unary { .. } => {}
//             Expression::Cast { .. } => {}
//             Expression::Binary { .. } => {}
//             Expression::Range { .. } => {}
//             Expression::InplaceAssignment { .. } => {}
//             Expression::Assignment { .. } => {}
//             Expression::IfElseExpression { .. } => {}
//             Expression::Block { .. } => {}
//             Expression::SelfExpression { .. } => {}
//             Expression::FnExpression { .. } => {}
//             Expression::StructInitializer { .. } => {}
//         }
//     }
// 
//     fn analyze_statement<SemanticsResult>(&mut self, statement: Statement)
//         -> Result<SemanticsResult, Self::Error>
//     {
//         match statement {
//             Statement::EmptyStatement { .. } => {}
//             Statement::LetStatement { .. } => {}
//             Statement::StaticStatement { .. } => {}
//             Statement::ConstStatement { .. } => {}
//             Statement::ExpressionStatement { .. } => {}
//             Statement::WhileStatement { .. } => {}
//             Statement::BreakStatement { .. } => {}
//             Statement::ContinueStatement { .. } => {}
//             Statement::FnStatement { .. } => {}
//             Statement::ReturnStatement { .. } => {}
//             Statement::DeferStatement { .. } => {}
//             Statement::StructStatement { .. } => {}
//             Statement::ImplStatement { .. } => {}
//             Statement::IfElseStatement { .. } => {}
//         }
//     }
// 
//     fn analyze_empty_statement<SemanticsResult: Default>(
//         &mut self,
//         parser: &mut Parser,
//     ) -> Result<SemanticsResult, Self::Error> {
//         Ok(SemanticsResult::default())
//     }
// }