// use crate::parser::semantics::traits::{AstContext, Semantics};
// use crate::parser::semantics::{SecondSemanticsPassContext};
// use crate::syntax::ast::{Assignment, AstNodeIndex, Binary, Call, Cast, FnStatement, Function, Method, Statement, Type, Unary};
// use crate::typing::literal_typing::{match_binary_op, NumericType};
// 
// #[derive(Debug, Clone)]
// pub struct TypeValidationContext {
// 
// }
// 
// impl TypeValidationContext {
//     pub fn new() -> Self {
//         Self {
// 
//         }
//     }
// }
// 
// impl Default for TypeValidationContext {
//     fn default() -> Self {
//         Self::new()
//     }
// }
// 
// impl TypeValidationContext {
//     #[inline(always)]
//     fn check_nodes_type_equality(&self, left: AstNodeIndex, right: AstNodeIndex) -> bool {
//         self.evaluated_types[left] == self.evaluated_types[right]
//     }
// }
// 
// impl AstContext for TypeValidationContext {}
// 
// pub struct TypeValidationSemantics;
// 
// impl Semantics<SecondSemanticsPassContext> for TypeValidationSemantics {
//     fn visit_binary(&self, binary: &Binary, context: &mut SecondSemanticsPassContext) {
//         self.visit_binary_default(binary, context);
// 
//         let left_type = context.type_validation.evaluted_types[binary.left.get_node_id()];
//         let right_type = context.type_validation.evaluated_types[binary.right.get_node_id()];
//         let Some(result_type) = match_binary_op(
//             binary.operator.token_type, left_type, right_type
//         ) else {
//             panic!(
//                 "Incompatible type pair for {:?} binary operator ({:?} {:?} {:?})",
//                 binary.operator.token_type,
//                 left_type, binary.operator.token_type,
//                 right_type
//             );
//         };
//         context.type_validation.evaluated_types.insert(binary.node_id, result_type);
//     }
// 
//     fn visit_unary(&self, unary: &Unary, context: &mut SecondSemanticsPassContext) {
//         self.visit_unary_default(unary, context);
//         let right_type = context.type_validation.get_expression_type(&unary.expression);
// 
//         let Some(result_type) = match_unary_op(unary.operator.token_type, right_type) else {
//             panic!(
//                 "Can't apply {} operator to type {}",
//                 unary.operator, right_type
//             );
//         };
// 
//         context.type_validation.set_expression_type(unary.node_id, result_type);
//     }
// 
//     fn visit_cast(&self, cast: &Cast, context: &mut SecondSemanticsPassContext) {
//         self.visit_cast_default(cast, context);
// 
//         let target_type = type_annotation_as_type(cast.target_type);
//         if !cast.is_reinterpret_cast {
//             let left_type = context.type_validation.get_expression_type(&cast.left);
// 
//             let result_type = match_cast_operator(left_type);
// 
//             if result_type != target_type {
//                 panic!(
//                     "Cannot cast {} to {} in {}",
//                     left_type, target_type, cast.token
//                 );
//             }
//         }
//         context.type_validation.set_expression_type(cast.node_id, result_type);
//     }
// 
//     fn visit_call(&self, call: &Call, context: &mut SecondSemanticsPassContext) {
//         self.visit_call_default(call, context);
//         let Some(function) = context.name_resolving.find_function_from_current_scope(call.node_id) else {
//             panic!("No function found for  call {:?}", call);
//         };
//         let callee_type = context.type_validation.get_expression_type(&call.callee);;
// 
//         if callee_type != NumericType::Function {
//             panic!(
//                 "Can't apply function call operator {} to non function value {}",
//                 call.token, callee_type
//             );
//         }
// 
//         // callee_type.
// 
//         let argument_types: Vec<_> = call.arguments.iter().map(|arg|
//             context.type_validation.get_expression_type(arg)
//         ).collect();
// 
//     }
// 
//     fn visit_assignment(&self, assignment: &Assignment, context: &mut SecondSemanticsPassContext) {
//         self.visit_assignment_default(assignment, context);
//         let left_type = context.type_validation.evaluted_types[assignment.lhs.get_node_id()];
//         let right_type = context.type_validation.evaluated_types[assignment.rhs.get_node_id()];
// 
//         if left_type != right_type {
//             panic!("Cannot assigne {} type to {} types", left_type, right_type);
//         }
// 
//         context.type_validation.evaluated_types.insert(assignment.node_id, left_type);
//     }
// 
//     fn visit_function_statement(&self, fn_statement: &FnStatement, function: &Function, context: &mut SecondSemanticsPassContext) {
//         // function.arguments.iter().for_each(|arg| {
//         //     arg.declared_type
//         // });
//         let return_exprs: Vec<_> = function.body
//             .iter()
//             .filter_map(
//                 |s| if let Statement::ReturnStatement(r) = s {
//                     Some(&r.expression)
//                 } else {
//                     None
//                 }
//             ).collect();
// 
//         if function.return_type.is_some() && return_exprs.is_empty() {
//             panic!("Function body doesn't have return statement {:?}", function)
//         }
// 
//         for return_expr in return_exprs.iter() {
//             match return_expr {
//                 Some(expr) => {
//                     if function.return_type.is_none() {
//                         panic!("void function can't have return statements with expression {:?}", return_expr);
//                     }
// 
//                     let expr_return_type = context.evaluated_types[expr.get_node_id()];
//                     if function_return_type != expr_return_type {
//                         panic!(
//                             "incompatible return type for function: function return type {}, but got {}",
//                             function_return_type, expr_return_type
//                         );
//                     }
//                 },
//                 None => {
//                     if function.return_type.is_some() {
//                         panic!("Can't return void from function with declared return type {:?}", function.return_type.as_ref().unwrap());
//                     }
//                 }
//             }
//         }
//     }
// 
//     fn visit_method_statement(&self, fn_statement: &FnStatement, method: &Method, context: &mut SecondSemanticsPassContext) {
//         let return_exprs: Vec<_> = method.body
//             .iter()
//             .filter_map(
//                 |s| if let Statement::ReturnStatement(r) = s {
//                     Some(&r.expression)
//                 } else {
//                     None
//                 }
//             ).collect();
// 
//         if method.return_type.is_some() && return_exprs.is_empty() {
//             panic!("Function body doesn't have return statement {:?}", method)
//         }
// 
//         for return_expr in return_exprs.iter() {
//             match return_expr {
//                 Some(expr) => {
//                     if method.return_type.is_none() {
//                         panic!("void function can't have return statements with expression {:?}", return_expr);
//                     }
// 
//                     let expr_return_type = context.evaluated_types[expr.get_node_id()];
//                     if function_return_type != expr_return_type {
//                         panic!(
//                             "incompatible return type for function: function return type {}, but got {}",
//                             function_return_type, expr_return_type
//                         );
//                     }
//                 },
//                 None => {
//                     if method.return_type.is_some() {
//                         panic!("Can't return void from function with declared return type {:?}", method.return_type.as_ref().unwrap());
//                     }
//                 }
//             }
//         }
//     }
// }