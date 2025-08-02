use std::collections::HashMap;
use crate::dev_assert;
use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::syntax::ast::{current_id, ArraySlice, ArrowAccess, Assignment, Ast, AstNode, AstNodeIndex, Binary, BlockExpression, BreakStatement, Call, Cast, ConstStatement, ContinueStatement, DeferStatement, DotAccess, Expression, ExpressionStatement, FnExpression, FnStatement, Function, Grouping, Identifier, IfElseExpression, IfElseStatement, ImplStatement, InplaceAssignment, LetStatement, LiteralNode, Method, Range, ReturnStatement, SelfExpression, Statement, StaticStatement, StructInitializer, StructStatement, Unary, WhileStatement};

#[derive(Debug, Clone)]
pub struct ParserContext<'a> {
    pub ast_nodes: HashMap<AstNodeIndex, AstRef<'a>>
}

impl Default for ParserContext<'_> {
    fn default() -> Self {
        Self {
            ast_nodes: HashMap::with_capacity(
                current_id().into()
            )
        }
    }
}

impl AstContext for ParserContext<'_> {}

pub struct FlattenTree;

#[derive(Debug, Clone)]
pub enum AstRef<'a> {
    Expression(&'a Expression),
    Statement(&'a Statement),
}

impl<'a> Semantics<ParserContext<'a>> for FlattenTree {
    fn visit_let_statement(
        &self,
        let_statement: &LetStatement,
        context: &mut ParserContext,
    )
    {
        self.visit_let_statement_default(let_statement, context);
    }

    fn visit_static_statement(
        &self,
        static_statement: &StaticStatement,
        context: &mut ParserContext,
    ) {
        self.visit_static_statement_default(static_statement, context);
    }

    fn visit_const_statement(
        &self,
        const_statement: &ConstStatement,
        context: &mut ParserContext,
    ) {
        self.visit_const_statement_default(const_statement, context);
    }

    fn visit_expression_statement(
        &self,
        expression_statement: &ExpressionStatement,
        context: &mut ParserContext,
    )
    {
        self.visit_expression_statement_default(expression_statement, context);
    }

    fn visit_while_statement(
        &self,
        while_statement: &WhileStatement,
        context: &mut ParserContext,
    )
    {
        self.visit_while_statement_default(while_statement, context)
    }

    fn visit_break_statement(
        &self,
        _break_statement: &BreakStatement,
        _context: &mut ParserContext,
    ) {}

    fn visit_continue_statement(
        &self,
        _continue_statement: &ContinueStatement,
        _context: &mut ParserContext,
    ) {}

    fn visit_function_statement(
        &self,
        fn_statement: &FnStatement,
        function: &Function,
        context: &mut ParserContext,
    )
    {
        self.visit_function_statement_default(fn_statement, function, context);
    }

    fn visit_method_statement(
        &self,
        fn_statement: &FnStatement,
        method: &Method,
        context: &mut ParserContext,
    )
    {
        self.visit_method_statement_default(fn_statement, method, context);
    }

    fn visit_fn_statement(
        &self,
        fn_statement: &FnStatement,
        context: &mut ParserContext,
    )
    {
        self.visit_fn_statement_default(fn_statement, context)
    }

    fn visit_return_statement(
        &self,
        return_statement: &ReturnStatement,
        context: &mut ParserContext,
    )
    {
        self.visit_return_statement_default(return_statement, context)
    }

    fn visit_defer_statement(
        &self,
        defer_statement: &DeferStatement,
        context: &mut ParserContext,
    ) {
        self.visit_defer_statement_default(defer_statement, context)
    }

    fn visit_struct_statement(
        &self,
        _struct_statement: &StructStatement,
        _context: &mut ParserContext,
    ) {}

    fn visit_impl_statement(
        &self,
        impl_statement: &ImplStatement,
        context: &mut ParserContext,
    )
    {
        self.visit_impl_statement_default(impl_statement, context)
    }

    fn visit_if_else_statement(
        &self,
        if_else_statement: &IfElseStatement,
        context: &mut ParserContext,
    )
    {
        self.visit_if_else_statement_default(
            if_else_statement, context
        );
    }

    fn visit_expression(
        &self, expression: &Expression, context: &mut ParserContext<'a>
    ) {
        self.visit_expression_default(expression, context);
        assert!(
            !context.ast_nodes.contains_key(&expression.get_node_id()),
            "Expression node_id {} is already defined\n{:?}",
            expression.get_node_id(),
            expression
        );

        context.ast_nodes.insert(expression.get_node_id(), AstRef::Expression(expression));
    }

    fn visit_grouping(
        &self,
        grouping: &Grouping,
        context: &mut ParserContext,
    )
    {
        self.visit_grouping_default(grouping, context);
    }

    fn visit_literal(
        &self,
        _literal: &LiteralNode,
        _context: &mut ParserContext,
    ) {}

    fn visit_identifier(
        &self,
        _identifier: &Identifier,
        _context: &mut ParserContext,
    ) {}

    fn visit_dot_access(
        &self,
        dot_access: &DotAccess,
        context: &mut ParserContext,
    )
    {
        self.visit_dot_access_default(dot_access, context);
    }

    fn visit_arrow_access(
        &self,
        arrow_access: &ArrowAccess,
        context: &mut ParserContext,
    )
    {
        self.visit_arrow_access_default(arrow_access, context);
    }

    fn visit_call(
        &self,
        call: &Call,
        context: &mut ParserContext,
    )
    {
        self.visit_call_default(call, context);
    }

    fn visit_array_slice(
        &self,
        slice: &ArraySlice,
        context: &mut ParserContext,
    )
    {
        self.visit_array_slice_default(slice, context);
    }

    fn visit_unary(
        &self,
        unary: &Unary,
        context: &mut ParserContext,
    )
    {
        self.visit_unary_default(unary, context);
    }

    fn visit_cast(
        &self,
        cast: &Cast,
        context: &mut ParserContext,
    )
    {
        self.visit_cast_default(cast, context);
    }

    fn visit_binary(
        &self,
        binary: &Binary,
        context: &mut ParserContext,
    )
    {
        self.visit_binary_default(binary, context);
    }

    fn visit_range(
        &self,
        range: &Range,
        context: &mut ParserContext,
    )
    {
        self.visit_range_default(range, context);
    }

    fn visit_inplace_assignment(
        &self,
        inplace_assignment: &InplaceAssignment,
        context: &mut ParserContext,
    )
    {
        self.visit_inplace_assignment_default(inplace_assignment, context);
    }

    fn visit_assignment(
        &self,
        assignment: &Assignment,
        context: &mut ParserContext,
    )
    {
        self.visit_assignment_default(assignment, context);
    }

    fn visit_if_else(
        &self,
        if_else: &IfElseExpression,
        context: &mut ParserContext,
    )
    {
        self.visit_if_else_default(if_else, context)
    }

    fn visit_block_expression(
        &self,
        block: &BlockExpression,
        context: &mut ParserContext,
    )
    {
        self.visit_block_expression_default(block, context)
    }

    fn visit_self(
        &self,
        _self: &SelfExpression,
        _context: &mut ParserContext,
    ) {}

    fn visit_fn_expression(
        &self,
        fn_expression: &FnExpression,
        context: &mut ParserContext,
    ) {
        self.visit_fn_expression_default(fn_expression, context);
    }

    fn visit_struct_initializer(
        &self,
        struct_initializer: &StructInitializer,
        context: &mut ParserContext,
    ) {
        self.visit_struct_initializer_default(struct_initializer, context);
    }
}