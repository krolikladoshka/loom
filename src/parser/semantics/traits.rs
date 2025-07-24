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
    fn visit_statement_default(&self, statement: &Statement, context: &mut SharedContext) {
        match statement {
            Statement::EmptyStatement { .. } => {}
            Statement::LetStatement(let_statement) =>
                {self.visit_let_statement(let_statement, context);}
            Statement::StaticStatement(static_statement) =>
                self.visit_static_statement(static_statement, context),
            Statement::ConstStatement(const_statement) =>
                self.visit_const_statement(const_statement, context),
            Statement::ExpressionStatement(expression_statement) =>
                self.visit_expression_statement(expression_statement, context),
            Statement::WhileStatement(while_statement) =>
                self.visit_while_statement(while_statement, context),
            Statement::BreakStatement(break_statement) =>
                self.visit_break_statement(break_statement, context),
            Statement::ContinueStatement(continue_statement) =>
                self.visit_continue_statement(continue_statement, context),
            Statement::FnStatement(fn_statement) =>
                self.visit_fn_statement(fn_statement, context),
            Statement::ReturnStatement(return_statement) =>
                self.visit_return_statement(return_statement, context),
            Statement::DeferStatement(defer_statement) =>
                self.visit_defer_statement(defer_statement, context),
            Statement::StructStatement(struct_statement) =>
                self.visit_struct_statement(struct_statement, context),
            Statement::ImplStatement(impl_statement) =>
                self.visit_impl_statement(impl_statement, context),
            Statement::IfElseStatement(if_else_statement) =>
                self.visit_if_else_statement(if_else_statement, context),
        }
    }
    
    fn visit_statement(&self, statement: &Statement, context: &mut SharedContext) {
        self.visit_statement_default(statement, context)
    }

    fn visit_next(&self, statement: &Statement, context: &mut SharedContext) {
        self.visit_statement(statement, context)
    }

    fn visit_all_statements(
        &self,
        statements: &Vec<Statement>,
        context: &mut SharedContext,
    )
    {
        for statement in statements {
            self.visit_statement(statement, context);
        }
    }
    
    fn visit_all_expressions(
        &self,
        expressions: &Vec<Expression>,
        context: &mut SharedContext,
    )
    {
        for expression in expressions {
            self.visit_expression(expression, context);
        }
    }

    fn empty_statement(&mut self, _semicolon: &Token, _context: &mut SharedContext) {}

    fn visit_let_statement_default(
        &self,
        let_statement: &LetStatement,
        context: &mut SharedContext,
    )
    {
        if let Some(initializer) = &let_statement.initializer {
            self.visit_expression(&initializer, context);
        }
    }
    fn visit_let_statement(
        &self,
        let_statement: &LetStatement,
        context: &mut SharedContext,
    )
    {
        self.visit_let_statement_default(let_statement, context);
    }

    fn visit_static_statement(
        &self,
        _static_statement: &StaticStatement,
        _context: &mut SharedContext,
    ) {}

    fn visit_const_statement(
        &self,
        _const_statement: &ConstStatement,
        _context: &mut SharedContext,
    ) {}

    fn visit_expression_statement_default(
        &self,
        expression_statement: &ExpressionStatement,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&expression_statement.expression, context);
    }
    
    fn visit_expression_statement(
        &self,
        expression_statement: &ExpressionStatement,
        context: &mut SharedContext,
    )
    {
        self.visit_expression_statement_default(expression_statement, context);
    }

    fn visit_while_statement_default(
        &self,
        while_statement: &WhileStatement,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&while_statement.condition, context);
        self.visit_all_statements(&while_statement.body, context);
    }
    fn visit_while_statement(
        &self,
        while_statement: &WhileStatement,
        context: &mut SharedContext,
    )
    {
        self.visit_while_statement_default(while_statement, context)
    }

    fn visit_break_statement(
        &self,
        _break_statement: &BreakStatement,
        _context: &mut SharedContext,
    ) {}

    fn visit_continue_statement(
        &self,
        _continue_statement: &ContinueStatement,
        _context: &mut SharedContext,
    ) {}

    fn visit_fn_statement(
        &self,
        _fn_statement: &FnStatement,
        _context: &mut SharedContext,
    ) {}
    
    fn visit_return_statement_default(
        &self,
        return_statement: &ReturnStatement,
        context: &mut SharedContext,
    )
    {
        if let Some(return_expression) = &return_statement.expression {
            self.visit_expression(return_expression, context);
        }
    }
    
    fn visit_return_statement(
        &self,
        return_statement: &ReturnStatement,
        context: &mut SharedContext,
    ) 
    {
        self.visit_return_statement_default(return_statement, context)
    }

    fn visit_defer_statement(
        &self,
        _defer_statement: &DeferStatement,
        _context: &mut SharedContext,
    ) {}

    fn visit_struct_statement(
        &self,
        _struct_statement: &StructStatement,
        _context: &mut SharedContext,
    ) {}

    fn visit_impl_statement(
        &self,
        _impl_statement: &ImplStatement,
        _context: &mut SharedContext,
    ) {}

    fn visit_if_else_statement_default(
        &self,
        if_else_statement: &IfElseStatement,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&if_else_statement.condition, context);
        self.visit_all_statements(&if_else_statement.then_branch, context);
        
        if let Some(else_statement) = &if_else_statement.else_branch {
            self.visit_all_statements(else_statement, context);
        }
    }
    
    fn visit_if_else_statement(
        &self,
        if_else_statement: &IfElseStatement,
        context: &mut SharedContext,
    ) 
    {
        self.visit_if_else_statement_default(
            if_else_statement, context
        );
    }

    fn visit_expression_default(&self, expression: &Expression, context: &mut SharedContext) {
        match expression {
            Expression::Grouping(grouping) =>
                self.visit_grouping(grouping, context),
            Expression::Literal(literal) =>
                self.visit_literal(literal, context),
            Expression::Identifier(identifier) =>
                self.visit_identifier(identifier, context),
            Expression::MethodCall { .. } => {}
            // self.analyze_method_call(method_call, context),
            Expression::DotSet { .. } => {}
            Expression::ArrowSet { .. } => {}
            Expression::DotAccess(dot_access) =>
                self.visit_dot_access(dot_access, context),
            Expression::ArrowAccess(arrow_access) =>
                self.visit_arrow_access(arrow_access, context),
            Expression::Call(call) =>
                self.visit_call(call, context),
            Expression::ArraySlice(array_slice) =>
                self.visit_array_slice(array_slice, context),
            Expression::Unary(unary) =>
                self.visit_unary(unary, context),
            Expression::Cast(cast) =>
                self.visit_cast(cast, context),
            Expression::Binary(binary) =>
                self.visit_binary(binary, context),
            Expression::Range(range) =>
                self.visit_range(range, context),
            Expression::InplaceAssignment(inplace_assignment) =>
                self.visit_inplace_assignment(inplace_assignment, context),
            Expression::Assignment(assignment) =>
                self.visit_assignment(assignment, context),
            Expression::IfElseExpression(if_else_expression) =>
                self.visit_if_else(if_else_expression, context),
            Expression::Block(block) =>
                self.visit_block_expression(block, context),
            Expression::SelfExpression(self_expression) =>
                self.visit_self(self_expression, context),
            Expression::FnExpression(fn_expression) =>
                self.visit_fn_expression(fn_expression, context),
            Expression::StructInitializer(struct_initializer) =>
                self.visit_struct_initializer(struct_initializer, context),
        }
    }

    fn visit_expression(&self, expression: &Expression, context: &mut SharedContext) {
        self.visit_expression_default(expression, context)
    }

    fn visit_grouping_default(
        &self,
        grouping: &Grouping,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&grouping.expression, context);
    }

    fn visit_grouping(
        &self,
        grouping: &Grouping,
        context: &mut SharedContext,
    )
    {
        self.visit_grouping_default(grouping, context);
    }

    fn visit_literal(
        &self,
        _literal: &LiteralNode,
        _context: &mut SharedContext,
    ) {}

    fn visit_identifier(
        &self,
        _identifier: &Identifier,
        _context: &mut SharedContext,
    ) {}

    fn visit_dot_access_default(
        &self,
        dot_access: &DotAccess,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&dot_access.object, context);
    }
    
    fn visit_dot_access(
        &self,
        dot_access: &DotAccess,
        context: &mut SharedContext,
    )
    {
        self.visit_dot_access_default(dot_access, context);
    }

    fn visit_arrow_access_default(
        &self,
        arrow_access: &ArrowAccess,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&arrow_access.pointer, context);
    }
    
    fn visit_arrow_access(
        &self,
        arrow_access: &ArrowAccess,
        context: &mut SharedContext,
    )
    {
        self.visit_arrow_access_default(arrow_access, context);
    }
    
    fn visit_call_default(&self, call: &Call, context: &mut SharedContext) {
        self.visit_expression(&call.callee, context);
        self.visit_all_expressions(&call.arguments, context);
    }

    fn visit_call(
        &self,
        call: &Call,
        context: &mut SharedContext,
    )
    {
        self.visit_call_default(call, context);
    }
    
    fn visit_array_slice_default(
        &self,
        array_slice: &ArraySlice,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&array_slice.array_expression, context);
        self.visit_expression(&array_slice.slice_expression, context);
    }
    
    fn visit_array_slice(
        &self,
        slice: &ArraySlice,
        context: &mut SharedContext,
    )
    {
        self.visit_array_slice_default(slice, context);
    }

    fn visit_unary_default(
        &self,
        unary: &Unary,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&unary.expression, context);
    }
    
    fn visit_unary(
        &self,
        unary: &Unary,
        context: &mut SharedContext,
    ) 
    {
        self.visit_unary_default(unary, context);
    }

    fn visit_cast_default(
        &self,
        cast: &Cast,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&cast.left, context);
    }
    
    fn visit_cast(
        &self,
        cast: &Cast,
        context: &mut SharedContext,
    ) 
    {
        self.visit_cast_default(cast, context);
    }
    
    fn visit_binary_default(
        &self,
        binary: &Binary,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&binary.left, context);
        self.visit_expression(&binary.right, context);
    }
    
    fn visit_binary(
        &self,
        binary: &Binary,
        context: &mut SharedContext,
    )
    {
        self.visit_binary_default(binary, context);
    }

    fn visit_range_default(
        &self,
        range: &Range,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&range.start, context);
        self.visit_expression(&range.end, context);
    }
    
    fn visit_range(
        &self,
        range: &Range,
        context: &mut SharedContext,
    )
    {
        self.visit_range_default(range, context);
    }

    fn visit_inplace_assignment_default(
        &self,
        inplace_assignment: &InplaceAssignment,
        context: &mut SharedContext,
    ) 
    {
        self.visit_expression(&inplace_assignment.lhs, context);
        self.visit_expression(&inplace_assignment.rhs, context);
    }
    
    fn visit_inplace_assignment(
        &self,
        inplace_assignment: &InplaceAssignment,
        context: &mut SharedContext,
    ) 
    {
        self.visit_inplace_assignment_default(inplace_assignment, context);
    }

    fn visit_assignment_default(
        &self,
        assignment: &Assignment,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&assignment.lhs, context);
        self.visit_expression(&assignment.rhs, context);
    }
    
    fn visit_assignment(
        &self,
        assignment: &Assignment,
        context: &mut SharedContext,
    ) 
    {
        self.visit_assignment_default(assignment, context);
    }
    
    fn visit_if_else_default(
        &self,
        if_else: &IfElseExpression,
        context: &mut SharedContext,
    )
    {
        self.visit_expression(&if_else.condition, context);
        self.visit_all_statements(&if_else.then_branch.statements, context);
        
        self.visit_block_expression(&if_else.then_branch, context);
        self.visit_block_expression(&if_else.else_branch, context);
    }
    
    fn visit_if_else(
        &self,
        if_else: &IfElseExpression,
        context: &mut SharedContext,
    )
    {
        self.visit_if_else_default(if_else, context)
    }

    fn visit_block_expression_default(
        &self,
        block_expression: &BlockExpression,
        context: &mut SharedContext,
    )
    {
        self.visit_all_statements(&block_expression.statements, context);
        if let Some(return_expression) = &block_expression.return_expression {
            self.visit_expression(return_expression, context);
        }
    }
    
    fn visit_block_expression(
        &self,
        block: &BlockExpression,
        context: &mut SharedContext,
    )
    {
        self.visit_block_expression_default(block, context)
    }

    fn visit_self(
        &self,
        _self: &SelfExpression,
        _context: &mut SharedContext,
    ) {}
    
    fn visit_fn_expression(
        &self,
        _fn_expression: &FnExpression,
        _context: &mut SharedContext,
    ) {}

    fn visit_struct_initializer(
        &self,
        _struct_initializer: &StructInitializer,
        _context: &mut SharedContext,
    ) {}
}
