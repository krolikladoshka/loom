use std::collections::HashMap;
use std::fmt::{Debug, Pointer};
use std::io::empty;
use std::ops::Deref;
use crate::parser::semantics::{TranspilerPassContext};
use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::syntax::ast::{ArrowAccess, Assignment, Ast, AstNode, AstNodeIndex, Binary, BreakStatement, Call, Cast, ContinueStatement, DotAccess, Expression, ExpressionStatement, FnStatement, Function, Grouping, Identifier, IfElseStatement, ImplFunction, ImplStatement, InplaceAssignment, LetStatement, LiteralNode, Method, PointerAnnotation, ReturnStatement, SelfExpression, Statement, StructInitializer, StructStatement, TypeAnnotation, TypeKind, TypedDeclaration, Unary, WhileStatement};
use crate::syntax::tokens::TokenType;
use crate::typing::literal_typing::{BuiltinType, FunctionType, PointerType, StructType, Type};

trait Transpiled<T: AstContext> {
    fn transpiled(&self, _context: &T) -> String
    where
        Self: Debug
    {
        format!("/* {:?} */", self)
    }
}

fn walk_pointer(pointer: &PointerType, result: &mut Vec<bool>) -> BuiltinType {
    result.push(pointer.mutable);
    match &pointer.inner_type {
        BuiltinType::Pointer(inner_pointer) => {
            walk_pointer(inner_pointer, result)
        },
        other => other.clone()
    }
}
impl Transpiled<TranspilerPassContext> for PointerType {
    fn transpiled(&self, context: &TranspilerPassContext) -> String {
        // *mut *const u32
        // u32 const * <mut> * <-
        let mut stack = Vec::new();
        let innermost_type = walk_pointer(self, &mut stack);

        let mut result = innermost_type.transpiled(context);

        for pointer_modifier in stack.iter().rev() {
            result.push_str(&format!(
                "{}*",
                if *pointer_modifier {
                    ""
                } else {
                    " const "
                }
            ));
        }

        result
    }
}

impl Transpiled<TranspilerPassContext> for FunctionType {}

impl Transpiled<TranspilerPassContext> for BuiltinType {
    fn transpiled(&self, context: &TranspilerPassContext) -> String
    {
        match &self {
            BuiltinType::Void => "void".to_string(),
            BuiltinType::Null => "const void* const".to_string(),
            BuiltinType::Bool => "bool".to_string(),
            BuiltinType::I8 => "i8".to_string(),
            BuiltinType::I16 => "i16".to_string(),
            BuiltinType::I32 => "i32".to_string(),
            BuiltinType::I64 => "i64".to_string(),
            BuiltinType::U8 => "u8".to_string(),
            BuiltinType::U16 => "u16".to_string(),
            BuiltinType::U32 => "u32".to_string(),
            BuiltinType::U64 => "u64".to_string(),
            BuiltinType::F32 => "f32".to_string(),
            BuiltinType::F64 => "f64".to_string(),
            BuiltinType::Usize => "usize".to_string(),
            BuiltinType::Char => "char".to_string(),
            BuiltinType::String => "char*".to_string(),
            BuiltinType::Pointer(pointer) => pointer.transpiled(context),
            BuiltinType::Function(function) => function.transpiled(context),
            BuiltinType::Struct(struct_type) => struct_type.name.clone(),
        }
    }
}

impl Transpiled<TranspilerPassContext> for Type {
    fn transpiled(&self, context: &TranspilerPassContext) -> String {
        match &self.ttype {
            BuiltinType::Pointer(pointer) => pointer.transpiled(context),
            other => format!(
                "{}{}",
                // todo: for pointer const should be places after
                if self.mutable { "" } else { "const " },
                other.transpiled(context),
            )
        }

    }
}

#[derive(Debug, Clone)]
pub struct CTranspilerContext {
    transpiled_stack: Vec<String>,
    transpile_stack_index: Vec<usize>,
    pub transpile_results: HashMap<AstNodeIndex, String>,
    pub header_result: String,
    pub result: String,
    level: usize,
}

impl Default for CTranspilerContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
struct TranspileHelper {
    indent_size: usize,
    level: usize,
    result: String,
}

impl TranspileHelper {
    pub fn from_context(context: &TranspilerPassContext) -> Self {
        Self {
            indent_size: 4,
            level: context.transpile.level,
            result: String::new(),
        }
    }

    pub fn block<T>(&mut self, block: T)
    where
        T: FnOnce(&mut Self)
    {
        self.level += 1;
        block(self);
        self.level -= 1
    }
    pub fn pad_value(&self) -> String {
        " ".repeat(self.level * self.indent_size)
    }

    pub fn indent_value(&self) -> String {
        " ".repeat((self.level + 1) * self.indent_size)
    }

    pub fn push(&mut self, value: &str) {
        self.result.push_str(
            value
        );
    }

    pub fn ln(&mut self) {
        self.result.push('\n');
    }

    pub fn pad(&mut self) {
        self.result.push_str(&self.pad_value());
    }

    pub fn space(&mut self) {
        self.result.push(' ');
    }

    pub fn indent(&mut self) {
        self.result.push_str(&self.indent_value());
    }

    pub fn wrap(&mut self) {
        self.result.push('\n');
        self.pad();
    }

    pub fn wrap_indent(&mut self) {
        self.result.push('\n');
        self.indent();
    }
    pub fn push_and_wrap(&mut self, value: &str) {
        self.push(value);
        self.wrap();
    }

    pub fn push_and_wrap_indent(&mut self, value: &str) {
        self.push(value);
        self.wrap_indent();
    }

    pub fn push_and_indent(&mut self, value: &str) {
        self.push(value);
        self.indent();
    }

    pub fn push_all_wrapped(&mut self, values: &[&str]) {
        values.iter().for_each(|value| {
            self.push(*value);
            self.wrap();
        });
    }

    pub fn push_all_padded<T: AsRef<str>>(&mut self, values: &[T], separator: &str) {
        if values.is_empty() {
            return;
        }

        self.pad();
        for i in 0..values.len() - 1 {
            self.push(values[i].as_ref());
            self.push(separator);
            self.wrap();

        }
        self.push(values.last().unwrap().as_ref());
    }

    pub fn push_all_indented<T: AsRef<str>>(&mut self, values: &[T], separator: &str) {
        if values.is_empty() {
            return;
        }

        self.indent();
        for i in 0..values.len() - 1 {
            self.push(values[i].as_ref());
            self.push(separator);
            self.wrap_indent()
        }
        self.push(values.last().unwrap().as_ref());
    }
}

impl Into<String> for TranspileHelper {
    fn into(self) -> String {
        self.result
    }
}

impl CTranspilerContext {
    pub fn new() -> Self {
        Self {
            transpiled_stack: vec![],
            transpile_stack_index: vec![],
            transpile_results: HashMap::new(),
            header_result: String::new(),
            result: Self::generate_main_c_file(),
            level: 0,
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

impl TranspilerPassContext {
    fn transpile_block<T>(&mut self, block: T)
    where
        T: FnOnce(&mut Self)
    {
        self.transpile.level += 1;

        block(self);

        self.transpile.level -= 1;
    }
}

// #[derive(Default)]
pub struct CTranspilerSemantics {
    indent: usize,
    wrap: bool,
}

impl Default for CTranspilerSemantics {
    fn default() -> Self {
        Self {
            indent: 0,
            wrap: false,
        }
    }
}

///// TODO!: I honestly can't understand whether the code is now shittier or more flexible
impl CTranspilerSemantics {
    fn add_main_c_file(&self, context: &mut CTranspilerContext) {}
    fn fallback_transpile(
        &self, statement: &Statement,
        context: &mut TranspilerPassContext,
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
        context: &mut TranspilerPassContext,
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
            .map(|s| self.get_transpiled_node(s, context).clone())
            .collect::<Vec<String>>()
    }

    fn transpile_all_exprs(
        &self, expressions: &Vec<Expression>,
        context: &mut TranspilerPassContext,
    ) -> Vec<String>
    {
        self.visit_all_expressions(expressions, context);

        expressions.iter()
            .map(|e| self.get_transpiled_node(e, context).clone())
            .collect::<Vec<String>>()
    }

    fn get_all_transpiled_statements(
        &self, statements: &[Statement],
        context: &mut TranspilerPassContext
    ) -> Vec<String>
    {
        statements.iter()
            .map(|s| self.get_transpiled_node(s, context).clone())
            .collect::<Vec<String>>()
    }

    fn get_all_transpiled_exprs(
        &self, exprs: &[Expression],
        context: &mut TranspilerPassContext
    ) -> Vec<String>
    {
        exprs.iter()
            .map(|s| self.get_transpiled_node(s, context).clone())
            .collect::<Vec<String>>()
    }

    #[inline(always)]
    fn try_get_transpiled<'a>(
        &self, ast: &Ast, context: &'a mut TranspilerPassContext,
    ) -> Option<&'a String> {
        context.transpile.get_transpile_result(&ast.get_node_id())
    }
    
    #[inline(always)]
    fn get_transpiled_node<'a, T: AstNode>(
        &self, stmt: &T, context: &'a mut TranspilerPassContext
    ) -> &'a String {
        context.transpile.get_transpile_result(&stmt.get_node_id()).unwrap()
    }

    #[inline(always)]
    fn try_get_transpiled_statement<'a>(
        &self, stmt: &Statement, context: &'a mut TranspilerPassContext
    ) -> Option<&'a String> {
        context.transpile.get_transpile_result(&stmt.get_node_id())
    }

    #[inline(always)]
    fn try_get_transpiled_expr<'a>(
        &self, expr: &Expression, context: &'a mut TranspilerPassContext
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
        context: &mut TranspilerPassContext,
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

    #[inline(always)]
    fn blank(&self) -> String {
        " ".repeat(self.indent)
    }
}




fn empty_builder() -> TranspileHelper {
    TranspileHelper::default()
}

fn builder(context: &TranspilerPassContext) -> TranspileHelper {
    TranspileHelper::from_context(context)
}

fn builder_with(init: String, context: &TranspilerPassContext) -> TranspileHelper {
    let mut builder = TranspileHelper::from_context(context);
    builder.push(&init);

    builder
}

impl TranspilerPassContext {
    fn find_struct_type_from_name<T: AstNode>(
        &self, node: &T, name: &String
    ) -> &StructType {
        let struct_node_id = self.name_scoping.find_struct_node_id_from_scope(
            node, name
        );

        self.type_validation.get_struct_type(&struct_node_id)
    }
}

impl Semantics<TranspilerPassContext> for CTranspilerSemantics {
    fn visit_statement_default(&self, statement: &Statement, context: &mut TranspilerPassContext) {
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

    fn visit_next(&self, statement: &Statement, context: &mut TranspilerPassContext) {
        self.visit_statement(statement, context);

        let transpiled = self.get_transpiled_node(statement, context).clone();
        context.transpile.result.push_str(
            transpiled.as_str()
        );
        context.transpile.result.push_str("\n");
    }

    fn visit_let_statement(
        &self, let_statement: &LetStatement, context: &mut TranspilerPassContext
    )
    {
        self.visit_let_statement_default(let_statement, context);

        let mut result = builder(context);
        result.pad();
        let variable_type = context.type_validation.get_node_type(let_statement);
        result.push(&variable_type.transpiled(context));
        result.space();
        result.push(&let_statement.name.lexeme);

        if let Some(initializer) = &let_statement.initializer {
            result.push(" = ");
            let transpiled_initializer = self.get_transpiled_node(
                initializer.as_ref(),
                context
            );

            result.push(transpiled_initializer);
        }

        result.push(";");

        context.transpile.set_transpile_result(
            let_statement.node_id,
            result.into()
        );
    }

    fn visit_expression_statement(
        &self, statement: &ExpressionStatement,
        context: &mut TranspilerPassContext
    )
    {
        self.visit_expression_statement_default(statement, context);
        
        let mut transpiled = self.get_transpiled_node(
            statement.expression.as_ref(), context
        ).clone();

        transpiled.push(';');

        context.transpile.set_transpile_result(
            statement.node_id,
            transpiled
        )
    }

    fn visit_while_statement(&self, while_statement: &WhileStatement, context: &mut TranspilerPassContext) {
        let mut result = builder(context);
        result.pad();
        result.push("while (");

        self.visit_while_statement_default(while_statement, context);

        self.visit_expression(&while_statement.condition, context);
        let condition_transpiled = self.get_transpiled_node(
            while_statement.condition.as_ref(), context
        );
        result.push(&condition_transpiled);
        result.push(") {");
        result.wrap_indent();

        result.block(|result| {
            let transpiled_body = self.transpile_all(&while_statement.body, context);
            result.push_all_padded(&transpiled_body, "");
            result.wrap();
        });
        result.push("}");

        context.transpile.set_transpile_result(
            while_statement.node_id,
            result.into()
        )
    }
    
    fn visit_break_statement(
        &self, break_statement: &BreakStatement,
        context: &mut TranspilerPassContext
    )
    {
        context.transpile.set_transpile_result(
            break_statement.node_id,
            "break;".to_string()
        );
    }

    fn visit_continue_statement(
        &self, continue_statement: &ContinueStatement,
        context: &mut TranspilerPassContext
    )
    {
        context.transpile.set_transpile_result(
            continue_statement.node_id,
            "continue;".to_string()
        )
    }

    fn visit_function_statement(
        &self,
        fn_statement: &FnStatement,
        function: &Function,
        context: &mut TranspilerPassContext
    ) {

        fn transpile_function_header(
            fn_statement: &FnStatement, function: &Function, context: &mut TranspilerPassContext
        ) -> String {
            let mut result = builder(context);
            result.pad();

            let function_type = context.type_validation.get_function_type(
                fn_statement
            ).unwrap();

            result.push(&function_type.return_type.transpiled(context));
            result.space();
            result.push(&function.name.lexeme);


            if function_type.arguments.len() > 0 {
                result.push("(\n");
                let function_arguments_length = function.arguments.len();

                let name_to_type: Vec<_> = function.arguments.iter()
                    .cloned()
                    .zip(function_type.arguments.iter().cloned())
                    .collect();

                // context.transpile_block(|ctx| {
                result.block(|result| {
                    let mut parameters = Vec::with_capacity(
                        function_arguments_length
                    );
                    for (parameter, parameter_type) in name_to_type {
                        let mut transpiled_parameter = parameter_type.transpiled(context);
                        transpiled_parameter.push_str(" ");
                        transpiled_parameter.push_str(&parameter.name.lexeme);

                        parameters.push(transpiled_parameter);
                    }

                    result.push_all_padded(&parameters, ",");
                });
                result.wrap();
                result.push(")");
            } else {
                result.push("()");
            }

            result.into()
        }


        let result = transpile_function_header(fn_statement, function, context);
        let mut result = builder_with(result, context);

        if function.body.len() > 0 {
            // context.transpile_block(|context| {
            result.space();
            result.push("{\n");

            let transpiled_body = self.transpile_all(&function.body, context);

            result.push_all_indented(&transpiled_body, "");
            result.wrap();
            result.push("}");
            // });
        } else {
            result.push("{}\n")
        }

        context.transpile.set_transpile_result(
            fn_statement.node_id,
            result.into()
        );
    }

    fn visit_method_statement(
        &self,
        fn_statement: &FnStatement,
        method: &Method,
        context: &mut TranspilerPassContext
    ) {

        fn transpile_function_header(
            fn_statement: &FnStatement, method: &Method, context: &mut TranspilerPassContext
        ) -> String {
            let mut result = builder(context);
            result.pad();

            let function_type = context.type_validation.get_function_type(
                fn_statement
            ).unwrap();

            result.push(&function_type.return_type.transpiled(context));
            result.space();
            result.push(&method.name.lexeme);

            let bound_type = &function_type.arguments[0];
            let self_parameter = format!("{} self", bound_type.transpiled(context));

            if function_type.arguments.len() > 0 {
                result.push("(\n");
                let function_arguments_length = method.arguments.len();

                let name_to_type: Vec<_> = method.arguments.iter()
                    .cloned()
                    .zip(function_type.arguments.iter().cloned())
                    .collect();

                // context.transpile_block(|ctx| {
                result.block(|result| {
                    let mut parameters = Vec::with_capacity(
                        function_arguments_length
                    );

                    parameters.push(self_parameter);

                    for (parameter, parameter_type) in name_to_type {
                        let mut transpiled_parameter = parameter_type.transpiled(context);
                        transpiled_parameter.push_str(" ");
                        transpiled_parameter.push_str(&parameter.name.lexeme);

                        parameters.push(transpiled_parameter);
                    }

                    result.push_all_padded(&parameters, ",");
                });
                result.wrap();
                result.push(")");
            } else {
                result.push(&format!("({})", self_parameter));
            }

            result.into()
        }


        let result = transpile_function_header(fn_statement, method, context);
        let mut result = builder_with(result, context);

        if method.body.len() > 0 {
            // context.transpile_block(|context| {
            result.space();
            result.push("{\n");

            let transpiled_body = self.transpile_all(&method.body, context);

            result.push_all_indented(&transpiled_body, "");
            result.wrap();
            result.push("}");
            // });
        } else {
            result.push("{}\n")
        }

        context.transpile.set_transpile_result(
            fn_statement.node_id,
            result.into()
        );
    }
    // fn visit_fn_statement(
    //     &self,
    //     statement: &FnStatement,
    //     context: &mut TranspilerPassContext
    // )
    // {
    //     // let mut result = String::new();
    //     let result = self.transpile_function(
    //         &statement.function, String::new(), context
    //     );
    //
    //     context.transpile.set_transpile_result(
    //         statement.node_id,
    //         result
    //     )
    // }

    fn visit_return_statement(
        &self, return_statement: &ReturnStatement,
        context: &mut TranspilerPassContext
    )
    {
        self.visit_return_statement_default(return_statement, context);
        let mut result = builder(context);
        // result.pad();
        result.push("return");

        if let Some(return_expression) = &return_statement.expression {
            result.space();

            let return_expr = self.get_transpiled_node(
                return_expression.as_ref(), context
            );

            result.push(return_expr);
        }

        result.push(";");
        context.transpile.set_transpile_result(
            return_statement.node_id,
            result.into()
        );
    }


    fn visit_struct_statement(
        &self, struct_statement: &StructStatement,
        context: &mut TranspilerPassContext
    )
    {
        fn build_struct_header(
            struct_statement: &StructStatement,
            context: &TranspilerPassContext
        ) -> String {
            let mut result = empty_builder();
            result.push(
                &format!(
                    "typedef struct {} {}",
                    &struct_statement.name.literal,
                    &struct_statement.name.literal
                )
            );

            result.into()
        }
        let struct_header = build_struct_header(struct_statement, context);
        context.transpile.header_result.push_str(&struct_header);
        context.transpile.header_result.push_str(";\n");

        let mut result = builder(context);

        result.pad();
        result.push(&format!("struct {} {{", &struct_statement.name.literal));
        // result.wrap_indent();
        result.ln();
        if struct_statement.fields.len() > 0 {
            result.block(|result| {
                let fields: Vec<_> = struct_statement.fields.iter()
                    .map(|field_declaration| {
                        let field_type = {
                            let field_type = context.type_validation.get_node_type(
                                &field_declaration.declared_type
                            );
                            field_type.ttype.transpiled(context)
                            // field_type.transpiled(context)
                        };

                        format!(
                            "{} {};",
                            field_type, field_declaration.name.literal
                        )
                    })
                    .collect();

                result.push_all_padded(&fields, "\n");
            });
            result.wrap();
        }
        result.push("}\n");

        context.transpile.set_transpile_result(
            struct_statement.node_id,
            result.into()
        );
    }

    fn visit_impl_statement(
        &self, impl_statement: &ImplStatement,
        context: &mut TranspilerPassContext
    )
    {
        let mut result = builder(context);
        result.pad();
        let define_block_name = format!(
            "{}_IMPL_BLOCK",
            impl_statement.implemented_type.literal.to_uppercase()
        );

        // result.push()
        // result.push(&format!("#define {}", define_block_name));


        result.block(|result| {
            let impl_prefix = impl_statement.implemented_type.literal.clone();
            // let mut transpiled_declarations = vec![];
            let mut top_level_statements = self.transpile_all(&impl_statement.top_level_statements, context);


            // for top_level_statement in &impl_statement.top_level_statements {
            //     self.visit_statement(top_level_statement, context);
            //     let transpiled_declaration = self.get_transpiled_statement(
            //         top_level_statement, context
            //     );
            //
            //     transpiled_declarations.push(transpiled_declaration.clone());
            // }

            for function in &impl_statement.functions {
                self.visit_fn_statement(function, context);
                let transpiled_function = self.get_transpiled_node(
                    function, context
                );
                top_level_statements.push(transpiled_function.clone());
            }
            result.push_all_padded(&top_level_statements, "");
        });

        // for function in &impl_statement.functions {
        //     let transpiled_function = self.transpile_function(
        //         &function.function, impl_prefix.clone(), context
        //     );
        //     transpiled_declarations.push(transpiled_function);
        // }
        //
        // let result = transpiled_declarations.join("\n\t");

        context.transpile.set_transpile_result(
            impl_statement.node_id,
            result.into()
        );
    }

    fn visit_if_else_statement(
        &self, if_else: &IfElseStatement, context: &mut TranspilerPassContext
    )
    {
        let mut result = builder(context);
        result.pad();
        result.push("if (");

        self.visit_expression(&if_else.condition, context);
        let transpiled_condition = self.get_transpiled_node(
            if_else.condition.as_ref(), context
        );
        result.push(transpiled_condition);
        result.push(") {");

        if if_else.then_branch.len() > 0 {
            result.wrap_indent();
            result.block(|result| {
                let then_body = self.transpile_all(
                    &if_else.then_branch, context
                );
                result.push_all_padded(&then_body, "\n");
                result.wrap();
            });
        }
        result.push("}");
        
        if let Some(else_branch) = &if_else.else_branch {
            result.push(" else {");

            if else_branch.len() > 0 {
                result.wrap_indent();
                result.block(|result| {
                    let else_branch = self.transpile_all(
                        else_branch, context
                    );
                    result.push_all_padded(&else_branch, "\n");
                    result.wrap();
                });
            }
            result.push("}\n");
        }

        context.transpile.set_transpile_result(
            if_else.node_id,
            result.into(),
        );
    }

    fn visit_expression(
        &self, expression: &Expression, context: &mut TranspilerPassContext
    )
    {
        use crate::syntax::ast::Expression::*;
        let mut result = if self.wrap {
            String::from("(")
        } else {
            String::from("")
        };

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
            StructInitializer(initializer) =>
                self.visit_struct_initializer(initializer, context),
            _ => {
                context.transpile.set_transpile_result(
                    expression.get_node_id(),
                    format!("/* {:?} */", expression)
                );
            }
        };

        let expr = self.get_transpiled_node(expression, context);
        result.push_str(expr);

        if self.wrap {
            result.push(')');
        }

        context.transpile.set_transpile_result(expression.get_node_id(), result);
    }

    fn visit_grouping(
        &self, grouping: &Grouping,
        context: &mut TranspilerPassContext
    )
    {
        self.visit_grouping_default(grouping, context);

        let result = self.get_transpiled_node(
            grouping.expression.as_ref(), context
        ).clone();

        context.transpile.set_transpile_result(
            grouping.node_id,
            format!(
                "({})", result
            )
        )
    }

    fn visit_literal(&self, literal: &LiteralNode, context: &mut TranspilerPassContext) {
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
        &self, identifier: &Identifier, context: &mut TranspilerPassContext
    )
    {
        context.transpile.set_transpile_result(
            identifier.node_id,
            identifier.name.literal.clone()
        )
    }

    fn visit_dot_access(
        &self, dot_access: &DotAccess, context: &mut TranspilerPassContext
    ) {
        self.visit_dot_access_default(dot_access, context);

        let mut result = String::with_capacity(3 + dot_access.name.lexeme.len());
        let pointer = self.get_transpiled_node(
            dot_access.object.as_ref(), context
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
        &self, arrow_access: &ArrowAccess, context: &mut TranspilerPassContext
    ) {
        self.visit_arrow_access_default(arrow_access, context);

        let mut result = String::with_capacity(4 + arrow_access.name.lexeme.len());
        let pointer = self.get_transpiled_node(
            arrow_access.pointer.as_ref(), context
        );
        result.push_str(pointer.as_str());
        result.push_str("->");
        result.push_str(arrow_access.name.lexeme.as_str());

        context.transpile.set_transpile_result(
            arrow_access.node_id,
            result
        );
    }

    fn visit_call(&self, call: &Call, context: &mut TranspilerPassContext) {
        self.visit_call_default(call, context);
        
        let mut result = String::with_capacity(4);

        let callee = self.get_transpiled_node(call.callee.as_ref(), context);
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

    fn visit_unary(&self, unary: &Unary, context: &mut TranspilerPassContext) {
        let mut result = match unary.operator.token_type{
            TokenType::Plus => String::from("+"),
            TokenType::Minus => String::from("-"),
            TokenType::Star => String::from("*"),
            TokenType::BinaryAnd | TokenType::MutRef => String::from("&"),
            TokenType::LogicalNot => String::from("!"),
            TokenType::BinaryInvert => String::from("~"),
            TokenType::Increment => String::from("++"),
            TokenType::Decrement => String::from("--"),
            _ => panic!("unknown unary operator {}", unary.operator),
        };

        self.visit_unary_default(unary, context);

        let expr = self.get_transpiled_node(unary.expression.as_ref(), context);

        result.push_str(expr.as_str());

        context.transpile.set_transpile_result(
            unary.node_id,
            result,
        )
    }

    fn visit_cast(&self, cast: &Cast, context: &mut TranspilerPassContext) {
        self.visit_cast_default(cast, context);

        let type_annotation = self.transpile_type_annotation(&cast.target_type);

        let cast_transpiled = self.get_transpiled_node(cast.left.as_ref(), context);
        let result = format!("({})({})", type_annotation, cast_transpiled);

        context.transpile.set_transpile_result(
            cast.node_id,
            result
        )
    }

    fn visit_binary(&self, binary: &Binary, context: &mut TranspilerPassContext) {
        self.visit_binary_default(binary, context);

        let mut left = self.get_transpiled_node(binary.left.as_ref(), context).clone();
        left.push(' ');
        left.push_str(&binary.operator.lexeme.to_string());
        left.push(' ');

        let right = self.get_transpiled_node(binary.right.as_ref(), context);
        left.push_str(right);

        context.transpile.set_transpile_result(
            binary.node_id,
            left
        )
    }

    fn visit_inplace_assignment(
        &self, inplace_assignment: &InplaceAssignment, context: &mut TranspilerPassContext
    )
    {
        self.visit_inplace_assignment_default(inplace_assignment, context);

        let lhs = self.get_transpiled_node(
            inplace_assignment.lhs.as_ref(), context
        ).clone();
        let rhs = self.get_transpiled_node(
            inplace_assignment.rhs.as_ref(), context
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
        &self, assignment: &Assignment, context: &mut TranspilerPassContext
    )
    {
        self.visit_assignment_default(assignment, context);

        let mut lhs = self.get_transpiled_node(assignment.lhs.as_ref(), context).clone();
        lhs.push_str(" = ");

        let rhs = self.get_transpiled_node(assignment.rhs.as_ref(), context);
        lhs.push_str(rhs.as_str());

        context.transpile.set_transpile_result(assignment.node_id, lhs);
    }

    fn visit_self(
        &self, self_expression: &SelfExpression,
        context: &mut TranspilerPassContext
    )
    {
        context.transpile.set_transpile_result(
            self_expression.node_id,
            "self".to_string()
        )
    }

    fn visit_struct_initializer(
        &self,
        struct_initializer: &StructInitializer,
        context: &mut TranspilerPassContext
    ) {
        self.visit_struct_initializer_default(struct_initializer, context);

        let mut result = builder(context);
        result.push(&struct_initializer.struct_name.literal.clone());

        result.push(" {");

        if struct_initializer.field_initializers.len() > 0 {
            result.block(|result| {
                result.wrap();
                let mut transpiled_fields = Vec::with_capacity(
                    struct_initializer.field_initializers.len()
                );

                for (field_name, field_initializer) in struct_initializer.field_initializers.iter() {
                    let transpiled_initializer = self.get_transpiled_node(field_initializer, context);
                    transpiled_fields.push(
                        format!(
                            ".{} = {}",
                            field_name.literal,
                            transpiled_initializer
                        )
                    );
                }

                result.push_all_padded(&transpiled_fields, ",");
                result.wrap();
            });
        }
        result.push("}");

        context.transpile.set_transpile_result(
            struct_initializer.node_id,
            result.into()
        );
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