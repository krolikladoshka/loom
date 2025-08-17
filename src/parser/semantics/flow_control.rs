use crate::parser::semantics::traits::Semantics;
use crate::parser::semantics::FirstSemanticsPassContext;
use crate::syntax::ast::{AstNode, AstNodeIndex, BreakStatement, ConstStatement, ContinueStatement, DeferStatement, FnStatement, Function, ImplFunction, ImplStatement, Method, ReturnStatement, SelfExpression, Statement, StaticStatement, WhileStatement};
use crate::syntax::lexer::Token;
use std::collections::{HashMap, LinkedList};
use std::hash::Hash;

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum BlockType {
    #[default]
    TopLevel,
    Function,
    Impl,
    Method,
    Loop,
    While,
    For,
}

impl BlockType {
    const LOOPS: &'static [Self] = &[
        BlockType::Loop,
        BlockType::While,
        BlockType::For,
    ];
}

#[derive(Debug, Clone, Default)]
pub struct FlowControlContext {
    temp_previous_block_type: BlockType,
    pub previous_block_type: BlockType,
    temp_current_block_type: BlockType,
    pub current_block_type: BlockType,
    
    impl_and_top_level_block_type: BlockType,
    function_scope_block_type: BlockType,

    closest_function_ast_node_id: Option<AstNodeIndex>,
    closest_impl_block_id: Option<AstNodeIndex>,
    pub self_to_impl_block: HashMap<AstNodeIndex, AstNodeIndex>,
    pub function_deferred_calls: HashMap<AstNodeIndex, LinkedList<AstNodeIndex>>,
    pub function_return_statements: HashMap<AstNodeIndex, Vec<AstNodeIndex>>,
    pub node_to_impl_block: HashMap<AstNodeIndex, AstNodeIndex>,
}

impl FlowControlContext {
    pub fn set_block_type(&mut self, block_type: BlockType) {
        self.temp_previous_block_type = self.previous_block_type;
        self.previous_block_type = self.current_block_type;
        self.temp_current_block_type = self.current_block_type;
        self.current_block_type = block_type;
    }

    pub fn restore_block_type(&mut self) {
        self.previous_block_type = self.temp_previous_block_type;
        self.current_block_type = self.temp_current_block_type;
    }
}


impl FirstSemanticsPassContext {
    fn scope_block_type<F>(&mut self, block_type: BlockType, block: F)
    where
        F: FnOnce(&mut Self),
    {
        let temp_previous_block_type = self.flow_control.previous_block_type;
        self.flow_control.previous_block_type = self.flow_control.current_block_type;

        let temp_current_block_type = self.flow_control.current_block_type;
        self.flow_control.current_block_type = block_type;

        block(self);

        self.flow_control.previous_block_type = temp_previous_block_type;
        self.flow_control.current_block_type = temp_current_block_type;
    }
}

#[derive(Default)]
pub struct FlowControlSemantics;

impl FlowControlSemantics {
    fn check_if_within_loops(
        &self, context: &FirstSemanticsPassContext,
        keyword: &Token,
    ) {
        if !BlockType::LOOPS.contains(&context.flow_control.current_block_type) {
            panic!("{} outside of loop statement", keyword);
        }
    }

    fn check_if_within_function(
        &self, context: &FirstSemanticsPassContext,
        keyword: &Token,
    )
    {
        if context.flow_control.function_scope_block_type != BlockType::Function &&
            context.flow_control.function_scope_block_type != BlockType::Method
        {
            panic!("{} statement outside of function", keyword);
        }
    }
    
    fn pin_defer_call_to_closest_function(
        &self,
        defer_statement: &DeferStatement,
        context: &mut FirstSemanticsPassContext,
    )
    {
        let Some(function_id) = context.flow_control.closest_function_ast_node_id else {
            panic!(
                "Tried to attach deferred call outside of function: defer statement {}",
                defer_statement.node_id
            );
        };

        context.flow_control.function_deferred_calls
            .entry(function_id)
            .or_default()
            .push_front(defer_statement.node_id);
    }

    fn pin_self_to_impl_block(
        &self,
        self_expression: &SelfExpression,
        context: &mut FirstSemanticsPassContext,
    )
    {
        let Some(impl_block_id) = context.flow_control.closest_impl_block_id else {
            panic!(
                "{} tried to attach self expression outside of impl block: self_expression statement {}",
                self_expression.token, self_expression.node_id
            );
        };

        context.flow_control.self_to_impl_block
            .insert(self_expression.node_id, impl_block_id);
        self.pin_to_impl_block(self_expression, context);
    }

    fn pin_to_impl_block<T: AstNode>(&self, node: &T, context: &mut FirstSemanticsPassContext) {
        let Some(impl_block_id) = context.flow_control.closest_impl_block_id else {
            panic!(
                "{} tried to attach node to an impl block outside of an impl block",
                node.get_node_id()
            );
        };

        if context.flow_control.self_to_impl_block.contains_key(&impl_block_id) {
            panic!(
                "{} node already attached to an impl block {}",
                node.get_node_id(), impl_block_id
            );
        }
        context.flow_control.node_to_impl_block
            .insert(node.get_node_id(), impl_block_id);
    }
    
    fn try_pin_to_impl_block<T: AstNode>(
        &self, node: &T, context: &mut FirstSemanticsPassContext
    ) {
        let Some(impl_block_id) = context.flow_control.closest_impl_block_id else {
            return;
        };

        if context.flow_control.self_to_impl_block.contains_key(&impl_block_id) {
            panic!(
                "{} node already attached to an impl block {}",
                node.get_node_id(), impl_block_id
            );
        }
        context.flow_control.node_to_impl_block
            .insert(node.get_node_id(), impl_block_id);
    }
}

impl Semantics<FirstSemanticsPassContext> for FlowControlSemantics {
    fn visit_static_statement(
        &self,
        static_statement: &StaticStatement,
        context: &mut FirstSemanticsPassContext
    ) {
        self.visit_static_statement_default(static_statement, context);
        self.try_pin_to_impl_block(static_statement, context);
    }

    fn visit_const_statement(
        &self,
        const_statement: &ConstStatement, 
        context: &mut FirstSemanticsPassContext
    ) {
        self.visit_const_statement_default(const_statement, context);
        self.try_pin_to_impl_block(const_statement, context);
    }

    fn visit_while_statement(
        &self,
        while_statement: &WhileStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        context.scope_block_type(
            BlockType::While,
            |ctx|
                self.visit_while_statement_default(while_statement, ctx)
        )
    }

    fn visit_break_statement(
        &self, break_statement: &BreakStatement, context: &mut FirstSemanticsPassContext
    )
    {
        self.check_if_within_loops(context, &break_statement.token);
    }

    fn visit_continue_statement(
        &self, continue_statement: &ContinueStatement, context: &mut FirstSemanticsPassContext
    )
    {
        self.check_if_within_loops(context, &continue_statement.token);
    }
    
    fn visit_function_statement(
        &self,
        fn_statement: &FnStatement,
        function: &Function,
        context: &mut FirstSemanticsPassContext
    )
    {
        let temp = context.flow_control.function_scope_block_type;
        context.flow_control.function_scope_block_type = BlockType::Function;
        
        context.scope_block_type(
            BlockType::Function,
            |ctx| self.visit_function_statement_default(
                fn_statement,
                function,
                ctx
            )
        );
        
        context.flow_control.function_scope_block_type = temp;
    }

    fn visit_method_statement(
        &self,
        fn_statement: &FnStatement,
        method: &Method,
        context: &mut FirstSemanticsPassContext
    )
    {
        let temp = context.flow_control.function_scope_block_type;
        context.flow_control.function_scope_block_type = BlockType::Method;
        
        context.scope_block_type(
            BlockType::Method,
            |ctx| self.visit_method_statement_default(
                fn_statement,
                method,
                ctx
            )
        );
        
        context.flow_control.function_scope_block_type = temp;
    }

    fn visit_fn_statement(
        &self, fn_statement: &FnStatement, context: &mut FirstSemanticsPassContext
    )
    {
        let previous_function = context.flow_control.closest_function_ast_node_id;
        context.flow_control.closest_function_ast_node_id = Some(fn_statement.node_id);
        self.visit_fn_statement_default(fn_statement, context);
        self.try_pin_to_impl_block(fn_statement, context);
        context.flow_control.closest_function_ast_node_id = previous_function;
    }

    fn visit_return_statement(
        &self, return_statement: &ReturnStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        self.check_if_within_function(context, &return_statement.token);
         
        context.flow_control.function_return_statements
            .entry(context.flow_control.closest_function_ast_node_id.unwrap())
            .or_default()
            .push(return_statement.node_id);
        
        self.visit_return_statement_default(return_statement, context);
        // TODO: mark a function ref in context
        // if return_statement.expression.is_none() && true {
        //     panic!("return statement without expression in non-unit function")
        // }
    }

    fn visit_defer_statement(
        &self, defer_statement: &DeferStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        self.check_if_within_function(context, &defer_statement.token);
        self.pin_defer_call_to_closest_function(defer_statement, context);
    }

    fn visit_impl_statement(
        &self, impl_statement: &ImplStatement, context: &mut FirstSemanticsPassContext
    )
    {
        let temp = context.flow_control.impl_and_top_level_block_type;
        context.flow_control.impl_and_top_level_block_type = BlockType::Impl;

        let prev_block_id = context.flow_control.closest_impl_block_id;
        context.flow_control.closest_impl_block_id = Some(impl_statement.node_id);

        context.scope_block_type(
            BlockType::Impl,
            |ctx| {
                for top_level_statement in &impl_statement.top_level_statements {
                    match top_level_statement {
                        Statement::StaticStatement(..) | Statement::ConstStatement(..) => {},
                        _ => panic!(
                            "Top level statements of impl block can be 
                            only static const or function declaration: {:?}",
                            top_level_statement
                        )
                    }
                }
                self.visit_all_statements(&impl_statement.top_level_statements, ctx);
                
                for fn_statement in &impl_statement.functions {
                    self.visit_fn_statement(fn_statement, ctx);
                }
            }
        );
        context.flow_control.closest_impl_block_id = prev_block_id;
        context.flow_control.impl_and_top_level_block_type = temp;
    }

    fn visit_self(
        &self,
        self_expression: &SelfExpression,
        context: &mut FirstSemanticsPassContext
    )
    {
        // todo: what if self is captured within nested closures?
        if context.flow_control.function_scope_block_type != BlockType::Method {
            panic!("{} outside of method", self_expression.token);
        }
        self.pin_self_to_impl_block(self_expression, context);
    }
}