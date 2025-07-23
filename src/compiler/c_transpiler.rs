use crate::parser::errors::ParserError;
use crate::parser::parser::Parser;
use crate::parser::semantics::traits::Semantics;
use crate::syntax::ast::{Assignment, Binary, Expression, ExpressionStatement, FnStatement, ImplFunction, Literal, PointerAnnotation, Statement, TypeAnnotation, TypeKind, TypedDeclaration, Unary, WhileStatement};

pub struct CTranspilerSemantics;

impl Semantics<> for CTranspilerSemantics {}

pub struct CTranspiler<'a> {
    parser: &'a mut Parser
}

impl<'a> CTranspiler<'a> {
    pub fn new(parser: &'a mut Parser) -> CTranspiler<'a> {
        CTranspiler { parser }
    }

    pub fn transpile(&mut self) -> String {
        // let mut transpiled = vec![];
        let mut result = String::with_capacity(1024);

        while let Some(transpiled) = self.transpile_next() {
            result.push_str(transpiled.as_str());
        }

        result
    }

    pub fn transpile_next(&mut self) -> Option<String> {
        match self.parser.parse_next() {
            Ok(t) => Some(self.transpile_statement(&t)),
            Err(ParserError::Eof) => None,
            Err(err) => {
                println!("{}", err);
                self.transpile_next()
                // panic!("");
            }
        }
    }

    fn transpile_statement(&self, statement: &Statement) -> String {
        use crate::syntax::ast::Statement::*;

        match statement {
            FnStatement(function) =>
                self.transpile_function(function),
            ExpressionStatement(expression) =>
                self.transpile_expression_statement(expression),
            WhileStatement(while_statement) => 
                self.transpile_while_statement(while_statement),
            _ => "".to_string()
            // _ => todo!("not implemented\n\t{:?}", statement)
        }
    }

    fn transpile_while_statement(
        &self, while_statement: &WhileStatement
    ) -> String {
        let mut result = String::from("while (");
        result.push_str(self.transpile_expression(&while_statement.condition).as_str());
        result.push_str(") {\n\t");
        
        let body = while_statement.body.iter()
            .map(|statement| self.transpile_statement(statement))
            .collect::<Vec<String>>()
            .join("\n\t\t");
        result.push_str(body.as_str());
        
        result.push_str("\n\t}\n");

        result
    }
    fn transpile_expression(&self, expression: &Expression) -> String {
        use crate::syntax::ast::Expression::*;
        let mut result = String::from("(");

        let expr = match expression {
            Assignment(assignment) => self.assignment(assignment),
            Binary(binary) => self.transpile_binary(binary),
            Unary(unary) => self.transpile_unary(unary),
            Literal(lit) => self.transpile_literal(lit),
            _ => format!("/* {:?} */", expression)
            // _ => todo!("not implemented\n\t{:?}", expression)
        };

        result.push_str(&expr);
        result.push(')');

        result
    }

    fn assignment(&self, assignment: &Assignment) -> String {
        let mut lhs = self.transpile_expression(&assignment.lhs);
        lhs.push_str(" = ");
        lhs.push_str(self.transpile_expression(&assignment.rhs).as_str());

        lhs
    }

    fn transpile_literal(&self, literal: &Literal) -> String {
        use crate::syntax::ast::Literal::*;

        match literal {
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
        }
    }

    fn transpile_unary(&self, unary: &Unary) -> String {
        let mut result = String::from(unary.operator.lexeme.as_str());

        result.push_str(self.transpile_expression(&unary.expression).as_str());

        result
    }
    fn transpile_binary(&self, binary: &Binary) -> String {
        let mut left = self.transpile_expression(&binary.left);
        left.push(' ');
        left.push_str(&binary.operator.lexeme.to_string());
        left.push(' ');

        left.push_str(self.transpile_expression(&binary.right).as_str());

        left
    }

    fn transpile_expression_statement(&self, statement: &ExpressionStatement) -> String {
        let mut transpiled = self.transpile_expression(&statement.expression);

        transpiled.push(';');

        transpiled
    }

    fn transpile_function(&self, statement: &FnStatement) -> String {
        let mut result = String::new();

        match &statement.function {
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

                let transpiled_body = function.body.iter()
                    .map(|stmt| self.transpile_statement(stmt))
                    .collect::<Vec<String>>();

                result.push_str(transpiled_body.join("\n\t").as_str());

                result.push_str("\n}\n\n");
            },
            _ => todo!("not implemented\n\t{:?}", statement)
        }

        result
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
}