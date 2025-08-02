use crate::parser::semantics::SecondSemanticsPassContext;
use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::syntax::ast::{ArraySlice, ArrowAccess, Assignment, AstNode, AstNodeIndex, Binary, BlockExpression, BreakStatement, Call, Cast, ConstStatement, ContinueStatement, DeferStatement, DotAccess, Expression, ExpressionStatement, FnExpression, FnStatement, Function, Grouping, Identifier, IfElseExpression, IfElseStatement, ImplFunction, ImplStatement, InplaceAssignment, LetStatement, LiteralNode, Method, Range, ReturnStatement, SelfExpression, Statement, StaticStatement, StructInitializer, StructStatement, TypeName, Unary, WhileStatement};

pub struct ScopeResolvingSemantics;

impl ScopeResolvingSemantics {

    pub fn get_ast_node(&self, context: &SecondSemanticsPassContext, ast_node: AstNodeIndex) {
    }
    pub fn get_node_scope_id<T>(
        &self,
        context: &SecondSemanticsPassContext,
        ast_node: &T
    )
        -> Option<usize>
    where
        T: AstNode
    {
        context
            .first_pass
            .name_scoping
            .local_scopes
            .scope_id_by_node_index.get(ast_node.get_node_id().0)
            .cloned()
    }

    #[inline(always)]
    fn expression_scope_id(
        &self, context: &SecondSemanticsPassContext, expression: &Expression
    ) -> Option<usize> {
        context
            .first_pass
            .name_scoping
            .local_scopes
            .scope_id_by_node_index.get(expression.get_node_id().0)
            .cloned()
    }

    #[inline(always)]
    fn statement_scope_id(
        &self, context: &SecondSemanticsPassContext, statement: &Statement
    ) -> Option<usize> {
        context
            .first_pass
            .name_scoping
            .local_scopes
            .scope_id_by_node_index.get(statement.get_node_id().0)
            .cloned()
    }

    fn expression_scope(
        &self, context: &SecondSemanticsPassContext, expression: &Expression
    ) -> Option<usize> {
        // let scope_id = self.expression_scope_id(context, expression)?;
        None
    }
}

impl AstContext for ScopeResolvingSemantics {}

impl Semantics<SecondSemanticsPassContext> for ScopeResolvingSemantics {
    fn visit_let_statement(
        &self,
        let_statement: &LetStatement,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_let_statement_default(let_statement, context);
    }

    fn visit_static_statement(
        &self,
        _static_statement: &StaticStatement,
        _context: &mut SecondSemanticsPassContext,
    ) {
        self.visit_static_statement_default();
    }

    fn visit_const_statement(
        &self,
        _const_statement: &ConstStatement,
        _context: &mut SecondSemanticsPassContext,
    ) {
        self.visit_const_statement_default();
    }

    fn visit_while_statement(
        &self,
        while_statement: &WhileStatement,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_while_statement_default(while_statement, context)
    }

    fn visit_function_statement(
        &self,
        fn_statement: &FnStatement,
        function: &Function,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_function_statement_default(fn_statement, function, context);
    }

    fn visit_method_statement(
        &self,
        fn_statement: &FnStatement,
        method: &Method,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_method_statement_default(fn_statement, method, context);
    }

    fn visit_return_statement(
        &self,
        return_statement: &ReturnStatement,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_return_statement_default(return_statement, context)
    }

    fn visit_defer_statement(
        &self,
        _defer_statement: &DeferStatement,
        _context: &mut SecondSemanticsPassContext,
    ) {}

    fn visit_struct_statement(
        &self,
        _struct_statement: &StructStatement,
        _context: &mut SecondSemanticsPassContext,
    ) {}

    fn visit_impl_statement(
        &self,
        impl_statement: &ImplStatement,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_impl_statement_default(impl_statement, context)
    }

    fn visit_if_else_statement(
        &self,
        if_else_statement: &IfElseStatement,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_if_else_statement_default(
            if_else_statement, context
        );
    }

    fn visit_literal(
        &self,
        _literal: &LiteralNode,
        _context: &mut SecondSemanticsPassContext,
    ) {}

    fn visit_identifier(
        &self,
        identifier: &Identifier,
        context: &mut SecondSemanticsPassContext,
    ) {
        let Some(scope_id) = self.get_node_scope_id(context, identifier) else {
            panic!(
                "identifier {} is not defined in name table", identifier.name
            );
        };

        if let None = context.first_pass.name_scoping.local_scopes.find_from_scope(
            scope_id,
            &identifier.name.lexeme
        ) {
            panic!(
                "identifier {} is not found in current scope",
                identifier.name
            );
        }
    }

    fn visit_dot_access(
        &self,
        dot_access: &DotAccess,
        context: &mut SecondSemanticsPassContext,
    )
    {
        /*
        struct A {
            name: i32
        }
        struct Ptr {
            struct_a: *A,
        }
        let struct_a = A { name: 3 };
        let struct_ptr = Ptr { struct_a: &struct_a };
        let ptr = &struct_ptr;

        ptr->struct_a->name
        *(ptr to struct)
        (*(ptr to struct)).struct_a
        (*((*ptr).struct_a)).name
         */
        // (*((*ptr).struct_a)).name
        // dot_access.object.ge
        self.visit_dot_access_default(dot_access, context);
    }

    fn visit_arrow_access(
        &self,
        arrow_access: &ArrowAccess,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_arrow_access_default(arrow_access, context);
    }

    fn visit_call(
        &self,
        call: &Call,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_call_default(call, context);
    }

    fn visit_array_slice(
        &self,
        slice: &ArraySlice,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_array_slice_default(slice, context);
    }

    fn visit_unary(
        &self,
        unary: &Unary,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_unary_default(unary, context);
    }

    fn visit_cast(
        &self,
        cast: &Cast,
        context: &mut SecondSemanticsPassContext,
    )
    {
        let Some(scope_id) = self.get_node_scope_id(context, &cast.target_type) else {
            panic!(
                "target type {:?} in type cast {} is not defined in name table",
                cast.target_type,
                cast.token
            );
        };
        if let None = context.first_pass.name_scoping.local_scopes.find_from_scope(
            scope_id,
            cast.target_type.get_type_name()
        ) {
            panic!(
                "target type {:?} in type cast {} is not found in current scope",
                cast.target_type,
                cast.token
            );
        }

        self.visit_cast_default(cast, context);
    }

    fn visit_binary(
        &self,
        binary: &Binary,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_binary_default(binary, context);
    }

    fn visit_range(
        &self,
        range: &Range,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_range_default(range, context);
    }

    fn visit_inplace_assignment(
        &self,
        inplace_assignment: &InplaceAssignment,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_inplace_assignment_default(inplace_assignment, context);
    }

    fn visit_assignment(
        &self,
        assignment: &Assignment,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_assignment_default(assignment, context);
    }

    fn visit_if_else(
        &self,
        if_else: &IfElseExpression,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_if_else_default(if_else, context)
    }

    fn visit_block_expression(
        &self,
        block: &BlockExpression,
        context: &mut SecondSemanticsPassContext,
    )
    {
        self.visit_block_expression_default(block, context)
    }

    fn visit_self(
        &self,
        _self: &SelfExpression,
        _context: &mut SecondSemanticsPassContext,
    ) {}

    fn visit_fn_expression(
        &self,
        _fn_expression: &FnExpression,
        _context: &mut SecondSemanticsPassContext,
    ) {}

    fn visit_struct_initializer(
        &self,
        struct_initializer: &StructInitializer,
        context: &mut SecondSemanticsPassContext,
    ) {
        let Some(scope_id) = self.get_node_scope_id(context, struct_initializer) else {
            panic!(
                "struct {} in struct initializer {} is not defined in name table",
                struct_initializer.struct_name.lexeme, struct_initializer.token
            );
        };

        let Some((_, struct_scope)) = context
            .first_pass
            .name_scoping
            .local_scopes
            .find_from_scope(
                scope_id, &struct_initializer.struct_name.lexeme
            )
        else {
            panic!(
                "struct {} in struct initializer {} is not found in current scope",
                struct_initializer.struct_name.lexeme, struct_initializer.token
            );
        };

        let Some(initialized_struct_id) = struct_scope.structs.get(
            &struct_initializer.struct_name.lexeme
        ) else {
            panic!(
                "struct {} in struct initializer {} is not found in current scope",
                struct_initializer.struct_name.lexeme, struct_initializer.token
            );
        };

        let Some(struct_node) = context.parser.get_struct(*initialized_struct_id) else {
            panic
        }

        for (field_name, field_initializer) in &struct_initializer.field_initializers {
            self.visit_expression(field_initializer, context);
            initialized_struct.
        }
    }
}
