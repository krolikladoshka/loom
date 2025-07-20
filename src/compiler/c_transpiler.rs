use crate::parser::errors::ParserError;
use crate::parser::parser::Parser;
use crate::syntax::ast::{Expression, ImplFunction, Literal, PointerAnnotation, Statement, TypeAnnotation, TypeKind, TypedDeclaration};
use crate::syntax::lexer::Token;
use crate::syntax::traits::TreePrint;

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
            FnStatement {
                token,
                function
            } => self.transpile_function(function),
            ExpressionStatement {
                expression
            } => self.transpile_expression_statement(expression),
            WhileStatement {
                condition, body,
                ..
            } => self.transpile_while_statement(condition, body),
            _ => "".to_string()
            // _ => todo!("not implemented\n\t{:?}", statement)
        }
    }

    fn transpile_while_statement(
        &self, condition: &Expression, body: &Vec<Statement>
    ) -> String {
        let mut result = String::from("while (");
        result.push_str(self.transpile_expression(condition).as_str());
        result.push_str(") {\n\t");
        
        let body = body.iter()
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
            Assignment {
                lhs, rhs,
                ..
            } => self.assignment(lhs, rhs),
            Binary {
                left,
                operator,
                right,
            } => self.transpile_binary(&operator, left, right),
            Unary {
                operator, expression,
                ..
            } => self.transpile_unary(operator, expression),
            Literal(lit) => self.transpile_literal(lit),
            _ => format!("/* {} */", expression.print_tree(0))
            // _ => todo!("not implemented\n\t{:?}", expression)
        };

        result.push_str(&expr);
        result.push(')');

        result
    }

    fn assignment(&self, lhs: &Expression, rhs: &Expression) -> String {
        let mut lhs = self.transpile_expression(lhs);
        lhs.push_str(" = ");
        lhs.push_str(self.transpile_expression(rhs).as_str());

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

    fn transpile_unary(&self, operator: &Token, expression: &Expression) -> String {
        let mut result = String::from(operator.lexeme.as_str());

        result.push_str(self.transpile_expression(expression).as_str());

        result
    }
    fn transpile_binary(
        &self, operator: &Token, left: &Expression, right: &Expression
    ) -> String {
        let mut left = self.transpile_expression(left);
        left.push(' ');
        left.push_str(&operator.lexeme.to_string());
        left.push(' ');

        left.push_str(self.transpile_expression(right).as_str());

        left
    }

    fn transpile_expression_statement(&self, expression: &Expression) -> String {
        let mut transpiled = self.transpile_expression(expression);

        transpiled.push(';');

        transpiled
    }

    fn transpile_function(&self, function: &ImplFunction) -> String {
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

                let transpiled_body = function.body.iter()
                    .map(|stmt| self.transpile_statement(stmt))
                    .collect::<Vec<String>>();

                result.push_str(transpiled_body.join("\n\t").as_str());

                result.push_str("\n}\n\n");
            },
            _ => todo!("not implemented\n\t{:?}", function)
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