use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::syntax::ast::{current_id, Ast, AstNode, AstNodeIndex, Context, Expression, ImplStatement, Statement, StructStatement, TypeAnnotation, TypedDeclaration};
use std::cell::Cell;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ParserContext {
    pub ast_nodes: HashMap<AstNodeIndex, Ast>
}

impl Default for ParserContext {
    fn default() -> Self {
        Self {
            ast_nodes: HashMap::with_capacity(
                current_id().into()
            )
        }
    }
}

impl AstContext for ParserContext {}

impl ParserContext {
    pub fn get_struct(&self, ast_node_index: AstNodeIndex) -> Option<&StructStatement> {
        let node = self.ast_nodes.get(&ast_node_index)?;

        match node {
            Ast::Statement(Statement::StructStatement(s)) =>
                Some(s),
            _ => None
        }
    }
}

#[derive(Default)]
pub struct FlattenTree;

impl<'a> Semantics<ParserContext> for FlattenTree {
    fn visit_statement(&self, statement: &Statement, context: &mut ParserContext) {
        self.visit_statement_default(statement, context);

        context.ast_nodes.insert(
            statement.get_node_id(),
            Ast::Statement(statement.clone())
        );
    }

    fn visit_impl_statement(
        &self,
        impl_statement: &ImplStatement,
        context: &mut ParserContext
    ) {
        self.visit_impl_statement_default(impl_statement, context);
        
        for function in &impl_statement.functions {
            context.ast_nodes.insert(
                function.get_node_id(),
                Ast::Statement(Statement::FnStatement(
                    function.clone()
                ))
            );
        }
    }
   
    fn visit_expression(
        &self, expression: &Expression, context: &mut ParserContext
    ) {
        self.visit_expression_default(expression, context);
        assert!(
            !context.ast_nodes.contains_key(&expression.get_node_id()),
            "Expression node_id {} is already defined\n{:?}",
            expression.get_node_id(),
            expression
        );
        let q = expression.get_node_id();

        context.ast_nodes.insert(
            expression.get_node_id(), Ast::Expression(expression.clone())
        );
    }

    fn visit_type_annotation(
        &self,
        type_annotation: &TypeAnnotation,
        context: &mut ParserContext
    ) {
        context.ast_nodes.insert(
            type_annotation.get_node_id(),
            Ast::Context(Context::TypeAnnotation(type_annotation.clone()))
        );
    }

    fn visit_typed_declaration(
        &self,
        type_declaration: &TypedDeclaration,
        context: &mut ParserContext
    ) {
        self.visit_type_annotation(
            &type_declaration.declared_type,
            context
        );
        context.ast_nodes.insert(
            type_declaration.get_node_id(),
            Ast::Context(Context::TypedDeclaration(type_declaration.clone()))
        );

    }
}