use std::collections::HashMap;
use crate::parser::semantics::FirstSemanticsPassContext;
use crate::parser::semantics::traits::Semantics;
use crate::syntax::ast::{ArrowAccess, Assignment, Ast, AstNodeIndex, Binary, BreakStatement, Call, Cast, ContinueStatement, DotAccess, Expression, ExpressionStatement, FnStatement, Grouping, Identifier, IfElseStatement, ImplFunction, ImplStatement, InplaceAssignment, LetStatement, LiteralNode, PointerAnnotation, ReturnStatement, SelfExpression, Statement, StructStatement, TypeAnnotation, TypeKind, TypedDeclaration, Unary, WhileStatement};

#[derive(Debug, Clone)]
pub struct CTranspilerContext {
    transpiled_stack: Vec<String>,
    transpile_stack_index: Vec<usize>,
    pub transpile_results: HashMap<AstNodeIndex, String>,
    pub result: String
}

impl Default for CTranspilerContext {
    fn default() -> Self {
        Self::new()
    }
}


impl CTranspilerContext {
    pub fn new() -> Self {
        Self {
            transpiled_stack: vec![],
            transpile_stack_index: vec![],
            transpile_results: HashMap::new(),
            result: Self::generate_main_c_file(),
        }
    }

    fn generate_main_c_file() -> String {
        let mut result = String::new();

        result.push_str("#include <stdlib.h>\n");
        result.push_str("#include \"loom_runtime.h\"\n");

        result
    }

    #[inline(always)]
    pub fn set_transpile_result(&mut self, idx: AstNodeIndex, result: String) {
        self.transpile_results.insert(idx, result);
    }

    #[inline(always)]
    pub fn get_transpile_result(&self, idx: &AstNodeIndex) -> Option<&String> {
        self.transpile_results.get(idx)
    }
}

pub struct CTranspilerSemantics;

///// TODO!: I honestly can't understand whether the code is now shittier or more flexible
impl CTranspilerSemantics {
    fn add_main_c_file(&self, context: &mut CTranspilerContext) {}
    fn fallback_transpile(
        &self, statement: &Statement,
        context: &mut FirstSemanticsPassContext,
    )
    {
        context.transpile.set_transpile_result(
            statement.get_node_id(),
            format!("/* NOT TRANSPILED: {:?} */", statement),
        );
    }

    #[inline(always)]
    fn transpile_all(
        &self, statements: &Vec<Statement>,
        context: &mut FirstSemanticsPassContext,
    ) -> Vec<String>
    {
        self.visit_all_statements(statements, context);

        // println!("{:?}", context.transpile.transpile_results.keys());

        // let mut result = vec![];
        //
        // for s in statements {
        //     println!("{}; {:?}", s.get_node_id().0, s);
        //     // result.push(s.to_string());
        //     let transpiled_statement = self.get_transpiled_statement(s, context);
        //     result.push(transpiled_statement.clone());
        // }
        //
        // result
        statements.iter()
            .map(|s| self.get_transpiled_statement(s, context).clone())
            .collect::<Vec<String>>()
    }

    fn transpile_all_exprs(
        &self, expressions: &Vec<Expression>,
        context: &mut FirstSemanticsPassContext,
    ) -> Vec<String>
    {
        self.visit_all_expressions(expressions, context);

        expressions.iter()
            .map(|e| self.get_transpiled_expr(e, context).clone())
            .collect::<Vec<String>>()
    }

    fn get_all_transpiled_statements(
        &self, statements: &[Statement],
        context: &mut FirstSemanticsPassContext
    ) -> Vec<String>
    {
        statements.iter()
            .map(|s| self.get_transpiled_statement(s, context).clone())
            .collect::<Vec<String>>()
    }

    fn get_all_transpiled_exprs(
        &self, exprs: &[Expression],
        context: &mut FirstSemanticsPassContext
    ) -> Vec<String>
    {
        exprs.iter()
            .map(|s| self.get_transpiled_expr(s, context).clone())
            .collect::<Vec<String>>()
    }

    #[inline(always)]
    fn try_get_transpiled<'a>(
        &self, ast: &Ast, context: &'a mut FirstSemanticsPassContext,
    ) -> Option<&'a String> {
        context.transpile.get_transpile_result(&ast.get_node_id())
    }
    
    #[inline(always)]
    fn get_transpiled<'a>(
        &self, ast: &Ast, context: &'a mut FirstSemanticsPassContext
    ) -> &'a String {
        context.transpile.get_transpile_result(&ast.get_node_id()).unwrap()
    }

    #[inline(always)]
    fn get_transpiled_statement<'a>(
        &self, stmt: &Statement, context: &'a mut FirstSemanticsPassContext
    ) -> &'a String {
        context.transpile.get_transpile_result(&stmt.get_node_id()).unwrap()
    }

    #[inline(always)]
    fn try_get_transpiled_statement<'a>(
        &self, stmt: &Statement, context: &'a mut FirstSemanticsPassContext
    ) -> Option<&'a String> {
        context.transpile.get_transpile_result(&stmt.get_node_id())
    }

    #[inline(always)]
    fn get_transpiled_expr<'a>(
        &self, expr: &Expression, context: &'a mut FirstSemanticsPassContext
    ) -> &'a String {
        context.transpile.get_transpile_result(&expr.get_node_id()).unwrap()
    }

    #[inline(always)]
    fn try_get_transpiled_expr<'a>(
        &self, expr: &Expression, context: &'a mut FirstSemanticsPassContext
    ) -> Option<&'a String> {
        context.transpile.get_transpile_result(&expr.get_node_id())
    }

    fn transpile_typed_declaration(&self, typed_declaration: &TypedDeclaration) -> String {
        let type_str = self.transpile_type_annotation(
            &typed_declaration.declared_type
        );

        vec![type_str.as_str(), typed_declaration.name.literal.as_str()].join(" ")
    }

    fn transpile_type_annotation(&self, type_annotation: &TypeAnnotation) -> String {
        let mut result = String::new();

        match &type_annotation.kind {
            TypeKind::Simple(t) => result.push_str(t.name.lexeme.as_str()),
            TypeKind::Pointer(pa) =>
                result.push_str(self.transpile_pointer_annotation(&pa).as_str())
        }

        result
    }

    fn transpile_pointer_annotation(&self, pointer: &PointerAnnotation) -> String {
        let mut result = String::new();
        // let mut stack = vec!["*"];
        let mut pointer_nesting = 1;

        let mut current = pointer;

        while let TypeKind::Pointer(pa) = current.inner_type.as_ref() {
            // stack.push("*");
            pointer_nesting += 1;
            current = pa;
        }

        let TypeKind::Simple(innermost_type) = current.inner_type.as_ref() else {
            unreachable!("Should never get here because of while let above")
        };

        result.push_str(innermost_type.name.literal.as_str());
        result.push_str("*".repeat(pointer_nesting).as_str());

        result
    }

    fn transpile_function(
        &self, function: &ImplFunction, prefix: String,
        context: &mut FirstSemanticsPassContext,
    ) -> String {
        // TODO: rewrite when namespacing model is chosen
        let mut result = String::new();

        match function {
            ImplFunction::Function(function) => {
                if let Some(rt) = &function.return_type {
                    result.push_str(self.transpile_type_annotation(&rt).as_str());
                } else {
                    result.push_str("void ");
                }

                result.push(' ');
                result.push_str(function.name.literal.as_str());
                result.push('(');

                let args_string = function.arguments.iter()
                    .map(
                        |arg| self.transpile_typed_declaration(arg)
                    )
                    .collect::<Vec<String>>()
                    .join(", ");
                result.push_str(args_string.as_str());

                result.push(')');
                result.push('\n');
                result.push_str("{\n\t");

                let transpiled_body = self.transpile_all(
                    &function.body,
                    context
                ).join("\n\t");

                result.push_str(transpiled_body.as_str());

                result.push_str("\n}\n\n");

                result
            },
            ImplFunction::Method(function) => {
                if let Some(rt) = &function.return_type {
                    result.push_str(self.transpile_type_annotation(&rt).as_str());
                } else {
                    result.push_str("void ");
                }

                result.push(' ');
                result.push_str(function.name.literal.as_str());
                result.push('(');

                if !function.is_mut_self {
                    result.push_str("const ")
                }
                let self_arg = format!("{}*const self", function.bound_type.literal);
                result.push_str(&self_arg);

                let args_string = function.arguments.iter()
                    .map(
                        |arg| self.transpile_typed_declaration(arg)
                    )
                    .collect::<Vec<String>>()
                    .join(", ");

                if !args_string.is_empty() {
                    result.push_str(args_string.as_str());
                }

                result.push(')');
                result.push('\n');
                result.push_str("{\n\t");

                let transpiled_body = self.transpile_all(
                    &function.body,
                    context
                ).join("\n\t");

                result.push_str(transpiled_body.as_str());

                result.push_str("\n}\n\n");

                result
            }
        }
    }
}

impl Semantics<FirstSemanticsPassContext> for CTranspilerSemantics {
    fn visit_statement(&self, statement: &Statement, context: &mut FirstSemanticsPassContext) {
        use Statement::*;

        match statement {
            FnStatement(function) =>
                self.visit_fn_statement(function, context),
            ExpressionStatement(expression) =>
                self.visit_expression_statement(expression, context),
            WhileStatement(while_statement) =>
                self.visit_while_statement(while_statement, context),
            LetStatement(let_statement) =>
                self.visit_let_statement(let_statement, context),
            IfElseStatement(if_else_statement) =>
                self.visit_if_else_statement(if_else_statement, context),
            ReturnStatement(return_statement) =>
                self.visit_return_statement(return_statement, context),
            StructStatement(struct_statement) =>
                self.visit_struct_statement(struct_statement, context),
            BreakStatement(break_statement) =>
                self.visit_break_statement(break_statement, context),
            ContinueStatement(continue_statement) =>
                self.visit_continue_statement(continue_statement, context),
            ImplStatement(impl_function) =>
                self.visit_impl_statement(impl_function, context),
            stmt => {
                self.fallback_transpile(stmt, context);
            }
            // _ => todo!("not implemented\n\t{:?}", statement)
        }
    }

    fn visit_next(&self, statement: &Statement, context: &mut FirstSemanticsPassContext) {
        self.visit_statement(statement, context);

        let transpiled = self.get_transpiled_statement(statement, context).clone();
        context.transpile.result.push_str(
            transpiled.as_str()
        );
        context.transpile.result.push_str("\n\n");
    }

    fn visit_let_statement(
        &self, let_statement: &LetStatement, context: &mut FirstSemanticsPassContext
    )
    {
        self.visit_let_statement_default(let_statement, context);
        
        let mut result = String::default();

        if let Some(t) = &let_statement.variable_type {
            let annotation = self.transpile_type_annotation(&t);
            result.push_str(annotation.as_str());
        } else {
            result.push_str("void*");
        }

        result.push(' ');
        result.push_str(let_statement.name.lexeme.as_str());
        
        
        if let Some(initializer) = &let_statement.initializer {
            result.push_str(" = ");

            let expression = self.get_transpiled_expr(initializer, context).clone();

            result.push_str(expression.as_str());
        }

        result.push(';');

        context.transpile.set_transpile_result(
            let_statement.node_id,
            result
        );
    }

    fn visit_expression_statement(
        &self, statement: &ExpressionStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        self.visit_expression_statement_default(statement, context);
        
        let mut transpiled = self.get_transpiled_expr(
            &statement.expression, context
        ).clone();

        transpiled.push(';');

        context.transpile.set_transpile_result(
            statement.node_id,
            transpiled
        )
    }

    fn visit_while_statement(&self, while_statement: &WhileStatement, context: &mut FirstSemanticsPassContext) {
        self.visit_while_statement_default(while_statement, context);
        
        let mut result = String::from("while (");
        let condition = self.get_transpiled_expr(
            &while_statement.condition, context
        );
        result.push_str(condition);
        result.push_str(") {\n\t");
        
        let body = self.get_all_transpiled_statements(
            &while_statement.body, context
        ).join("\n\t");

        result.push_str(body.as_str());
        result.push_str("\n\t}\n");

        context.transpile.set_transpile_result(
            while_statement.node_id,
            result
        )
    }
    
    fn visit_break_statement(
        &self, break_statement: &BreakStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        context.transpile.set_transpile_result(
            break_statement.node_id,
            "break;".to_string()
        );
    }

    fn visit_continue_statement(
        &self, continue_statement: &ContinueStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        context.transpile.set_transpile_result(
            continue_statement.node_id,
            "continue;".to_string()
        )
    }
    fn visit_fn_statement(
        &self,
        statement: &FnStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        // let mut result = String::new();
        let result = self.transpile_function(
            &statement.function, String::new(), context
        );

        context.transpile.set_transpile_result(
            statement.node_id,
            result
        )
    }

    fn visit_return_statement(
        &self, return_statement: &ReturnStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        self.visit_return_statement_default(return_statement, context);
        
        let mut result = String::from("return");

        if let Some(return_expression) = &return_statement.expression {
            let return_expr = self.get_transpiled_expr(
                &return_expression, context
            );

            result.push(' ');
            result.push_str(return_expr.as_str());
        }

        result.push(';');
        context.transpile.set_transpile_result(
            return_statement.node_id,
            result
        );
    }


    fn visit_struct_statement(
        &self, struct_statement: &StructStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        let mut result = format!(
            "typedef struct {} {{\n\t",
            struct_statement.name.literal
        );

        let fields = struct_statement.fields
            .iter()
            .map(|td| self.transpile_typed_declaration(td) + ";")
            .collect::<Vec<String>>()
            .join("\n\t");
        
        result.push_str(fields.as_str());
        result.push_str(
            format!("\n}} {};\n", struct_statement.name.literal).as_str()
        );

        context.transpile.set_transpile_result(
            struct_statement.node_id,
            result
        );
    }

    fn visit_impl_statement(
        &self, impl_statement: &ImplStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        let impl_prefix = impl_statement.implemented_type.literal.clone();
        let mut transpiled_declarations = vec![];
        for top_level_statement in &impl_statement.top_level_statements {
            self.visit_statement(top_level_statement, context);
            let transpiled_declaration = self.get_transpiled_statement(
                top_level_statement, context
            );

            transpiled_declarations.push(transpiled_declaration.clone());
        }

        for function in &impl_statement.functions {
            let transpiled_function = self.transpile_function(
                &function.function, impl_prefix.clone(), context
            );
            transpiled_declarations.push(transpiled_function);
        }

        let result = transpiled_declarations.join("\n\t");

        context.transpile.set_transpile_result(
            impl_statement.node_id,
            result
        );
    }

    fn visit_if_else_statement(
        &self, if_else: &IfElseStatement, context: &mut FirstSemanticsPassContext
    )
    {
        self.visit_if_else_statement_default(if_else, context);
        
        let condition = self.get_transpiled_expr(&if_else.condition, context).clone();
        let then_branch = self.get_all_transpiled_statements(&if_else.then_branch, context)
            .join("\n\t");
        
        let mut result = format!("if ({}) {{\n\t{}\n\t}}", condition, then_branch);
        
        if let Some(else_branch) = &if_else.else_branch {
            let else_branch = self.get_all_transpiled_statements(
                else_branch, context
            ).join("\n\t");

            result.push_str(format!(" else {{\n\t{}\n\t}}", else_branch).as_str());
        }

        context.transpile.set_transpile_result(
            if_else.node_id,
            result,
        );
    }

    fn visit_expression(
        &self, expression: &Expression, context: &mut FirstSemanticsPassContext
    )
    {
        use crate::syntax::ast::Expression::*;
        let mut result = String::from("(");

        match expression {
            Assignment(assignment) => {
                self.visit_assignment(assignment, context);
            },
            InplaceAssignment(assignment) =>
                self.visit_inplace_assignment(assignment, context),
            Binary(binary) =>
                self.visit_binary(binary, context),
            Unary(unary) =>
                self.visit_unary(unary, context),
            Literal(lit) =>
                self.visit_literal(lit, context),
            Call(call) =>
                self.visit_call(call, context),
            ArrowAccess(access) =>
                self.visit_arrow_access(access, context),
            DotAccess(dot_access) =>
                self.visit_dot_access(dot_access, context),
            Identifier(identifier) =>
                self.visit_identifier(identifier, context),
            Cast(cast) =>
                self.visit_cast(cast, context),
            SelfExpression(expression) =>
                self.visit_self(expression, context),
            Grouping(grouping) =>
                self.visit_grouping(grouping, context),
            _ => {
                context.transpile.set_transpile_result(
                    expression.get_node_id(),
                    format!("/* {:?} */", expression)
                );
            }
        };

        let expr = self.get_transpiled_expr(expression, context);
        result.push_str(expr);
        result.push(')');

        context.transpile.set_transpile_result(expression.get_node_id(), result);
    }

    fn visit_grouping(
        &self, grouping: &Grouping,
        context: &mut FirstSemanticsPassContext
    )
    {
        self.visit_grouping_default(grouping, context);

        let result = self.get_transpiled_expr(
            &grouping.expression, context
        ).clone();

        context.transpile.set_transpile_result(
            grouping.node_id,
            format!(
                "({})", result
            )
        )
    }

    fn visit_literal(&self, literal: &LiteralNode, context: &mut FirstSemanticsPassContext) {
        use crate::syntax::ast::Literal::*;

        let lit_str = match &literal.literal {
            U8 {
                value,
                ..
            } => value.to_string(),
            U16 {
                value,
                ..
            } => value.to_string(),
            U32 {
                value,
                ..
            } => value.to_string(),
            U64 {
                value,
                ..
            } => value.to_string(),
            I8 {
                value,
                ..
            } => value.to_string(),
            I16 {
                value,
                ..
            } => value.to_string(),
            I32 {
                value,
                ..
            } => value.to_string(),
            I64 {
                value,
                ..
            } => value.to_string(),
            Bool {
                value,
                ..
            } => if *value { "true".to_string() } else { "false".to_string() },
            Char {
                value,
                ..
            } => format!("'{}'", value.to_string()),
            String {
                value,
                ..
            } => format!("\"{}\"", value),
            _ => todo!("not implemented\n\t{:?}", literal)
        };

        context.transpile.set_transpile_result(
            literal.node_id,
            lit_str
        );
    }

    fn visit_identifier(
        &self, identifier: &Identifier, context: &mut FirstSemanticsPassContext
    )
    {
        context.transpile.set_transpile_result(
            identifier.node_id,
            identifier.name.literal.clone()
        )
    }

    fn visit_dot_access(
        &self, dot_access: &DotAccess, context: &mut FirstSemanticsPassContext
    ) {
        self.visit_dot_access_default(dot_access, context);

        let mut result = String::with_capacity(3 + dot_access.name.lexeme.len());
        let pointer = self.get_transpiled_expr(
            &dot_access.object, context
        );
        result.push_str(pointer.as_str());
        result.push_str(".");
        result.push_str(dot_access.name.lexeme.as_str());

        context.transpile.set_transpile_result(
            dot_access.node_id,
            result
        );
    }

    fn visit_arrow_access(
        &self, arrow_access: &ArrowAccess, context: &mut FirstSemanticsPassContext
    ) {
        self.visit_arrow_access_default(arrow_access, context);

        let mut result = String::with_capacity(4 + arrow_access.name.lexeme.len());
        let pointer = self.get_transpiled_expr(
            &arrow_access.pointer, context
        );
        result.push_str(pointer.as_str());
        result.push_str("->");
        result.push_str(arrow_access.name.lexeme.as_str());

        context.transpile.set_transpile_result(
            arrow_access.node_id,
            result
        );
    }

    fn visit_call(&self, call: &Call, context: &mut FirstSemanticsPassContext) {
        self.visit_call_default(call, context);
        
        let mut result = String::with_capacity(4);

        let callee = self.get_transpiled_expr(call.callee.as_ref(), context);
        result.push_str(callee);
        result.push('(');
        
        let args = self.get_all_transpiled_exprs(&call.arguments, context);
        result.push_str(args.join(", ").as_str());
        result.push(')');

        context.transpile.set_transpile_result(
            call.node_id,
            result
        )
    }

    fn visit_unary(&self, unary: &Unary, context: &mut FirstSemanticsPassContext) {
        let mut result = String::from(unary.operator.lexeme.as_str());

        self.visit_unary_default(unary, context);

        let expr = self.get_transpiled_expr(&unary.expression, context);

        result.push_str(expr.as_str());

        context.transpile.set_transpile_result(
            unary.node_id,
            result,
        )
    }

    fn visit_cast(&self, cast: &Cast, context: &mut FirstSemanticsPassContext) {
        self.visit_cast_default(cast, context);

        let type_annotation = self.transpile_type_annotation(&cast.target_type);

        let cast_transpiled = self.get_transpiled_expr(&cast.left, context);
        let result = format!("({})({})", type_annotation, cast_transpiled);

        context.transpile.set_transpile_result(
            cast.node_id,
            result
        )
    }

    fn visit_binary(&self, binary: &Binary, context: &mut FirstSemanticsPassContext) {
        self.visit_binary_default(binary, context);

        let mut left = self.get_transpiled_expr(&binary.left, context).clone();
        left.push(' ');
        left.push_str(&binary.operator.lexeme.to_string());
        left.push(' ');

        let right = self.get_transpiled_expr(&binary.right, context);
        left.push_str(right);

        context.transpile.set_transpile_result(
            binary.node_id,
            left
        )
    }

    fn visit_inplace_assignment(
        &self, inplace_assignment: &InplaceAssignment, context: &mut FirstSemanticsPassContext
    )
    {
        self.visit_inplace_assignment_default(inplace_assignment, context);

        let lhs = self.get_transpiled_expr(
            &inplace_assignment.lhs, context
        ).clone();
        let rhs = self.get_transpiled_expr(
            &inplace_assignment.rhs, context
        );

        let mut result = String::new();
        result.push_str(lhs.as_str());
        result.push_str(format!(" {} ",inplace_assignment.operator.lexeme).as_str());
        result.push_str(rhs);

        context.transpile.set_transpile_result(
            inplace_assignment.node_id,
            result
        );
    }

    fn visit_assignment(
        &self, assignment: &Assignment, context: &mut FirstSemanticsPassContext
    )
    {
        self.visit_assignment_default(assignment, context);

        let mut lhs = self.get_transpiled_expr(&assignment.lhs, context).clone();
        lhs.push_str(" = ");

        let rhs = self.get_transpiled_expr(&assignment.rhs, context);
        lhs.push_str(rhs.as_str());

        context.transpile.set_transpile_result(assignment.node_id, lhs);
    }

    fn visit_self(
        &self, self_expression: &SelfExpression,
        context: &mut FirstSemanticsPassContext
    )
    {
        context.transpile.set_transpile_result(
            self_expression.node_id,
            "self".to_string()
        )
    }
}

// pub struct CTranspiler<'a> {
//     parser: &'a mut Parser
// }
//
// impl<'a> CTranspiler<'a> {
//     pub fn new(parser: &'a mut Parser) -> CTranspiler<'a> {
//         CTranspiler { parser }
//     }
//
//     pub fn transpile(&mut self) -> String {
//         // let mut transpiled = vec![];
//         let mut result = String::with_capacity(1024);
//
//         while let Some(transpiled) = self.transpile_next() {
//             result.push_str(transpiled.as_str());
//         }
//
//         result
//     }
//
//     pub fn transpile_next(&mut self) -> Option<String> {
//         match self.parser.parse_next() {
//             Ok(t) => Some(self.transpile_statement(&t)),
//             Err(ParserError::Eof) => None,
//             Err(err) => {
//                 println!("{}", err);
//                 self.transpile_next()
//                 // panic!("");
//             }
//         }
//     }
//
//     fn transpile_statement(&self, statement: &Statement) -> String {
//         use crate::syntax::ast::Statement::*;
//
//         match statement {
//             FnStatement(function) =>
//                 self.transpile_function(function),
//             ExpressionStatement(expression) =>
//                 self.transpile_expression_statement(expression),
//             WhileStatement(while_statement) =>
//                 self.transpile_while_statement(while_statement),
//             _ => "".to_string()
//             // _ => todo!("not implemented\n\t{:?}", statement)
//         }
//     }
//
//     fn transpile_while_statement(
//         &self, while_statement: &WhileStatement
//     ) -> String {
//         let mut result = String::from("while (");
//         result.push_str(self.transpile_expression(&while_statement.condition).as_str());
//         result.push_str(") {\n\t");
//
//         let body = while_statement.body.iter()
//             .map(|statement| self.transpile_statement(statement))
//             .collect::<Vec<String>>()
//             .join("\n\t\t");
//         result.push_str(body.as_str());
//
//         result.push_str("\n\t}\n");
//
//         result
//     }
//
//     fn transpile_literal(&self, literal: &Literal) -> String {
//         use crate::syntax::ast::Literal::*;
//
//         match literal {
//             U8 {
//                 value,
//                 ..
//             } => value.to_string(),
//             U16 {
//                 value,
//                 ..
//             } => value.to_string(),
//             U32 {
//                 value,
//                 ..
//             } => value.to_string(),
//             U64 {
//                 value,
//                 ..
//             } => value.to_string(),
//             I8 {
//                 value,
//                 ..
//             } => value.to_string(),
//             I16 {
//                 value,
//                 ..
//             } => value.to_string(),
//             I32 {
//                 value,
//                 ..
//             } => value.to_string(),
//             I64 {
//                 value,
//                 ..
//             } => value.to_string(),
//             Bool {
//                 value,
//                 ..
//             } => if *value { "true".to_string() } else { "false".to_string() },
//             Char {
//                 value,
//                 ..
//             } => format!("'{}'", value.to_string()),
//             String {
//                 value,
//                 ..
//             } => format!("\"{}\"", value),
//             _ => todo!("not implemented\n\t{:?}", literal)
//         }
//     }
//
//     fn transpile_unary(&self, unary: &Unary) -> String {
//         let mut result = String::from(unary.operator.lexeme.as_str());
//
//         result.push_str(self.transpile_expression(&unary.expression).as_str());
//
//         result
//     }
//     fn transpile_binary(&self, binary: &Binary) -> String {
//         let mut left = self.transpile_expression(&binary.left);
//         left.push(' ');
//         left.push_str(&binary.operator.lexeme.to_string());
//         left.push(' ');
//
//         left.push_str(self.transpile_expression(&binary.right).as_str());
//
//         left
//     }
//
//     fn transpile_expression_statement(&self, statement: &ExpressionStatement) -> String {
//         let mut transpiled = self.transpile_expression(&statement.expression);
//
//         transpiled.push(';');
//
//         transpiled
//     }
//
//     fn transpile_function(&self, statement: &FnStatement) -> String {
//         let mut result = String::new();
//
//         match &statement.function {
//             ImplFunction::Function(function) => {
//                 if let Some(rt) = &function.return_type {
//                     result.push_str(self.transpile_type_annotation(&rt).as_str());
//                 } else {
//                     result.push_str("void ");
//                 }
//
//                 result.push(' ');
//                 result.push_str(function.name.literal.as_str());
//                 result.push('(');
//
//                 let args_string = function.arguments.iter()
//                     .map(
//                         |arg| self.transpile_typed_declaration(arg)
//                     )
//                     .collect::<Vec<String>>()
//                     .join(", ");
//                 result.push_str(args_string.as_str());
//
//                 result.push(')');
//                 result.push('\n');
//                 result.push_str("{\n\t");
//
//                 let transpiled_body = function.body.iter()
//                     .map(|stmt| self.transpile_statement(stmt))
//                     .collect::<Vec<String>>();
//
//                 result.push_str(transpiled_body.join("\n\t").as_str());
//
//                 result.push_str("\n}\n\n");
//             },
//             _ => todo!("not implemented\n\t{:?}", statement)
//         }
//
//         result
//     }
//
//     fn transpile_typed_declaration(&self, typed_declaration: &TypedDeclaration) -> String {
//         let type_str = self.transpile_type_annotation(
//             &typed_declaration.declared_type
//         );
//
//         vec![type_str.as_str(), typed_declaration.name.literal.as_str()].join(" ")
//     }
//
//     fn transpile_type_annotation(&self, type_annotation: &TypeAnnotation) -> String {
//         let mut result = String::new();
//
//         match &type_annotation.kind {
//             TypeKind::Simple(t) => result.push_str(t.name.lexeme.as_str()),
//             TypeKind::Pointer(pa) =>
//                 result.push_str(self.transpile_pointer_annotation(&pa).as_str())
//         }
//
//         result
//     }
//
//     fn transpile_pointer_annotation(&self, pointer: &PointerAnnotation) -> String {
//         let mut result = String::new();
//         // let mut stack = vec!["*"];
//         let mut pointer_nesting = 1;
//
//         let mut current = pointer;
//
//         while let TypeKind::Pointer(pa) = current.inner_type.as_ref() {
//             // stack.push("*");
//             pointer_nesting += 1;
//             current = pa;
//         }
//
//         let TypeKind::Simple(innermost_type) = current.inner_type.as_ref() else {
//             unreachable!("Should never get here because of while let above")
//         };
//
//         result.push_str(innermost_type.name.literal.as_str());
//         result.push_str("*".repeat(pointer_nesting).as_str());
//
//         result
//     }
// }