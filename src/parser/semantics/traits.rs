// TODO: do something with formatter and linter to wrap uses to new lines or use use stmt per line
use crate::syntax::ast::{ArraySlice, ArrowAccess, Assignment, Binary, BlockExpression, BreakStatement, Call, Cast, ConstStatement, ContinueStatement, DeferStatement, DotAccess, Expression, ExpressionStatement, FnExpression, FnStatement, Grouping, Identifier, IfElseExpression, IfElseStatement, ImplStatement, InplaceAssignment, LetStatement, Literal, LiteralNode, Range, ReturnStatement, SelfExpression, Statement, StaticStatement, StructInitializer, StructStatement, Unary, WhileStatement};
use crate::syntax::lexer::Token;

pub trait AstContext: Default {
    type Output: Default;
}

pub trait Semantics<SharedContext>
where
    SharedContext: AstContext
{
    fn analyze_statement_default(&self, statement: &Statement, context: &mut SharedContext) {
        match statement {
            Statement::EmptyStatement { .. } => {}
            Statement::LetStatement(let_statement) =>
                {self.analyze_let_statement(let_statement, context);}
            Statement::StaticStatement(static_statement) =>
                self.analyze_static_statement(static_statement, context),
            Statement::ConstStatement(const_statement) =>
                self.analyze_const_statement(const_statement, context),
            Statement::ExpressionStatement(expression_statement) =>
                self.analyze_expression_statement(expression_statement, context),
            Statement::WhileStatement(while_statement) =>
                self.analyze_while_statement(while_statement, context),
            Statement::BreakStatement(break_statement) =>
                self.analyze_break_statement(break_statement, context),
            Statement::ContinueStatement(continue_statement) =>
                self.analyze_continue_statement(continue_statement, context),
            Statement::FnStatement(fn_statement) =>
                self.analyze_fn_statement(fn_statement, context),
            Statement::ReturnStatement(return_statement) =>
                self.analyze_return_statement(return_statement, context),
            Statement::DeferStatement(defer_statement) =>
                self.analyze_defer_statement(defer_statement, context),
            Statement::StructStatement(struct_statement) =>
                self.analyze_struct_statement(struct_statement, context),
            Statement::ImplStatement(impl_statement) =>
                self.analyze_impl_statement(impl_statement, context),
            Statement::IfElseStatement(if_else_statement) =>
                self.analyze_if_else_statement(if_else_statement, context),
        }
    }
    
    fn analyze_statement(&self, statement: &Statement, context: &mut SharedContext) {
        self.analyze_statement_default(statement, context)
    }

    fn analyze_next(&self, statement: &Statement, context: &mut SharedContext) {
        self.analyze_statement(statement, context)
    }

    fn analyze_all_statements(
        &self,
        statements: &Vec<Statement>,
        context: &mut SharedContext,
    )
    {
        for statement in statements {
            self.analyze_statement(statement, context);
        }
    }
    
    fn analyze_all_expressions(
        &self,
        expressions: &Vec<Expression>,
        context: &mut SharedContext,
    )
    {
        for expression in expressions {
            self.analyze_expression(expression, context);
        }
    }

    fn empty_statement(&mut self, _semicolon: &Token, _context: &mut SharedContext) {}

    fn analyze_let_statement(
        &self,
        _let_statement: &LetStatement,
        _context: &mut SharedContext,
    ) -> SharedContext::Output {
        SharedContext::Output::default()
    }

    fn analyze_static_statement(
        &self,
        _static_statement: &StaticStatement,
        _context: &mut SharedContext,
    ) {}

    fn analyze_const_statement(
        &self,
        _const_statement: &ConstStatement,
        _context: &mut SharedContext,
    ) {}

    fn analyze_expression_statement(
        &self,
        _expression_statement: &ExpressionStatement,
        _context: &mut SharedContext,
    ) {}

    fn analyze_while_statement_default(
        &self,
        while_statement: &WhileStatement,
        context: &mut SharedContext,
    )
    {
        self.analyze_expression(&while_statement.condition, context);
        self.analyze_all_statements(&while_statement.body, context);
    }
    fn analyze_while_statement(
        &self,
        _while_statement: &WhileStatement,
        _context: &mut SharedContext,
    )
    {
        self.analyze_while_statement_default(_while_statement, _context)
    }

    fn analyze_break_statement(
        &self,
        _break_statement: &BreakStatement,
        _context: &mut SharedContext,
    ) {}

    fn analyze_continue_statement(
        &self,
        _continue_statement: &ContinueStatement,
        _context: &mut SharedContext,
    ) {}

    fn analyze_fn_statement(
        &self,
        _fn_statement: &FnStatement,
        _context: &mut SharedContext,
    ) {}

    fn analyze_return_statement(
        &self,
        _return_statement: &ReturnStatement,
        _context: &mut SharedContext,
    ) {}

    fn analyze_defer_statement(
        &self,
        _defer_statement: &DeferStatement,
        _context: &mut SharedContext,
    ) {}

    fn analyze_struct_statement(
        &self,
        _struct_statement: &StructStatement,
        _context: &mut SharedContext,
    ) {}

    fn analyze_impl_statement(
        &self,
        _impl_statement: &ImplStatement,
        _context: &mut SharedContext,
    ) {}

    fn analyze_if_else_statement(
        &self,
        _if_else_statement: &IfElseStatement,
        _context: &mut SharedContext,
    ) {}

    fn analyze_expression_default(&self, expression: &Expression, context: &mut SharedContext) {
        match expression {
            Expression::Grouping(grouping) =>
                self.analyze_grouping(grouping, context),
            Expression::Literal(literal) =>
                self.analyze_literal(literal, context),
            Expression::Identifier(identifier) =>
                self.analyze_identifier(identifier, context),
            Expression::MethodCall { .. } => {}
            // self.analyze_method_call(method_call, context),
            Expression::DotSet { .. } => {}
            Expression::ArrowSet { .. } => {}
            Expression::DotAccess(dot_access) =>
                self.analyze_dot_access(dot_access, context),
            Expression::ArrowAccess(arrow_access) =>
                self.analyze_arrow_access(arrow_access, context),
            Expression::Call(call) =>
                self.analyze_call(call, context),
            Expression::ArraySlice(array_slice) =>
                self.analyze_array_slice(array_slice, context),
            Expression::Unary(unary) =>
                self.analyze_unary(unary, context),
            Expression::Cast(cast) =>
                self.analyze_cast(cast, context),
            Expression::Binary(binary) =>
                self.analyze_binary(binary, context),
            Expression::Range(range) =>
                self.analyze_range(range, context),
            Expression::InplaceAssignment(inplace_assignment) =>
                self.analyze_inplace_assignment(inplace_assignment, context),
            Expression::Assignment(assignment) =>
                self.analyze_assignment(assignment, context),
            Expression::IfElseExpression(if_else_expression) =>
                self.analyze_if_else(if_else_expression, context),
            Expression::Block(block) =>
                self.analyze_block_expression(block, context),
            Expression::SelfExpression(self_expression) =>
                self.analyze_self(self_expression, context),
            Expression::FnExpression(fn_expression) =>
                self.analyze_fn_expression(fn_expression, context),
            Expression::StructInitializer(struct_initializer) =>
                self.analyze_struct_initializer(struct_initializer, context),
        }
    }

    fn analyze_expression(&self, expression: &Expression, context: &mut SharedContext) {
        self.analyze_expression_default(expression, context)
    }

    fn analyze_grouping(
        &self,
        grouping: &Grouping,
        context: &mut SharedContext,
    )
    {
        self.analyze_expression(&grouping.expression, context);
    }

    fn analyze_literal(
        &self,
        _literal: &LiteralNode,
        _context: &mut SharedContext,
    ) {}

    fn analyze_identifier(
        &self,
        _identifier: &Identifier,
        _context: &mut SharedContext,
    ) {}

    fn analyze_dot_access(
        &self,
        _dot_access: &DotAccess,
        _context: &mut SharedContext,
    ) {}

    fn analyze_arrow_access(
        &self,
        _arrow_access: &ArrowAccess,
        _context: &mut SharedContext,
    ) {}

    fn analyze_call(
        &self,
        _call: &Call,
        _context: &mut SharedContext,
    ) {}

    fn analyze_array_slice(
        &self,
        _slice: &ArraySlice,
        _context: &mut SharedContext,
    ) {}

    fn analyze_unary(
        &self,
        _unary: &Unary,
        _context: &mut SharedContext,
    ) {}

    fn analyze_cast(
        &self,
        _cast: &Cast,
        _context: &mut SharedContext,
    ) {}

    fn analyze_binary(
        &self,
        _binary: &Binary,
        _context: &mut SharedContext,
    ) {}

    fn analyze_range(
        &self,
        _range: &Range,
        _context: &mut SharedContext,
    ) {}

    fn analyze_inplace_assignment(
        &self,
        _inplace_assignment: &InplaceAssignment,
        _context: &mut SharedContext,
    ) {}

    fn analyze_assignment(
        &self,
        _assignment: &Assignment,
        _context: &mut SharedContext,
    ) {}
    
    fn analyze_if_else_default(
        &self,
        if_else: &IfElseExpression,
        context: &mut SharedContext,
    )
    {
        self.analyze_expression(&if_else.condition, context);
        self.analyze_all_statements(&if_else.then_branch.statements, context);
        
        self.analyze_block_expression(&if_else.then_branch, context);
        self.analyze_block_expression(&if_else.else_branch, context);
    }
    fn analyze_if_else(
        &self,
        if_else: &IfElseExpression,
        context: &mut SharedContext,
    )
    {
        self.analyze_if_else_default(if_else, context)
    }

    fn analyze_block_expression_default(
        &self,
        block_expression: &BlockExpression,
        context: &mut SharedContext,
    )
    {
        self.analyze_all_statements(&block_expression.statements, context);
        if let Some(return_expression) = &block_expression.return_expression {
            self.analyze_expression(return_expression, context);
        }
    }
    
    fn analyze_block_expression(
        &self,
        block: &BlockExpression,
        context: &mut SharedContext,
    )
    {
        self.analyze_block_expression_default(block, context)
    }

    fn analyze_self(
        &self,
        _self: &SelfExpression,
        _context: &mut SharedContext,
    ) {}

    fn analyze_fn_expression(
        &self,
        _fn_expression: &FnExpression,
        _context: &mut SharedContext,
    ) {}

    fn analyze_struct_initializer(
        &self,
        _struct_initializer: &StructInitializer,
        _context: &mut SharedContext,
    ) {}
}
