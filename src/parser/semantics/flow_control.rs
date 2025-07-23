use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::syntax::ast::{BreakStatement, ContinueStatement, DeferStatement, FnStatement, ImplFunction, ImplStatement, ReturnStatement, SelfExpression, WhileStatement};
use crate::syntax::lexer::Token;

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


#[derive(Debug, Clone, Default)]
pub struct FirstSemanticsPassContext {
    // pub resolved_variables: HashMap<String, Identifier>,
    // pub scopes: Vec<Scope>,
    // pub transformed_ast: HashMap<usize, TransformedAstNode>
    flow_control: FlowControlContext,
}

impl AstContext for FirstSemanticsPassContext { type Output = (); }
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
        if context.flow_control.current_block_type != BlockType::Function &&
            context.flow_control.current_block_type != BlockType::Method
        {
            panic!("{} statement outside of function", keyword);
        }
    }

    fn analyze_impl_function(
        &self, function: &ImplFunction, context: &mut FirstSemanticsPassContext
    )
    {
        let block_type = match function {
            ImplFunction::Function(_)  => BlockType::Function,
            ImplFunction::Method(_) => BlockType::Method,
        };

        if block_type == BlockType::Method &&
            context.flow_control.current_block_type != BlockType::Impl
        {
            panic!("Can't define a method outside of impl block");
        }

        context.scope_block_type(
            block_type,
            |ctx| {
                match &function {
                    ImplFunction::Function(function) => {
                        self.analyze_all_statements(&function.body, ctx);
                    },
                    ImplFunction::Method(method) => {
                        self.analyze_all_statements(&method.body, ctx);
                    }
                }
            }
        )
    }
}

impl Semantics<FirstSemanticsPassContext> for FlowControlSemantics {
    fn analyze_while_statement(
        &self,
        while_statement: &WhileStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        context.scope_block_type(
            BlockType::While,
            |ctx|
                self.analyze_while_statement_default(while_statement, ctx)
        )
    }

    fn analyze_break_statement(
        &self, break_statement: &BreakStatement, context: &mut FirstSemanticsPassContext)
    {
        self.check_if_within_loops(context, &break_statement.token);
    }

    fn analyze_continue_statement(
        &self, continue_statement: &ContinueStatement, context: &mut FirstSemanticsPassContext)
    {
        self.check_if_within_loops(context, &continue_statement.token);
    }

    fn analyze_fn_statement(
        &self, fn_statement: &FnStatement, context: &mut FirstSemanticsPassContext
    )
    {
        self.analyze_impl_function(&fn_statement.function, context)
    }

    fn analyze_return_statement(
        &self, return_statement: &ReturnStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        self.check_if_within_function(context, &return_statement.token);
        // TODO: mark a function ref in context
        // if return_statement.expression.is_none() && true {
        //     panic!("return statement without expression in non-unit function")
        // }
    }

    fn analyze_defer_statement(
        &self, defer_statement: &DeferStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        self.check_if_within_function(context, &defer_statement.token);
    }

    fn analyze_impl_statement(
        &self, impl_statement: &ImplStatement, context: &mut FirstSemanticsPassContext
    )
    {
        context.scope_block_type(
            BlockType::Impl,
            |ctx| {
                impl_statement.functions
                    .iter()
                    .for_each(|function| {
                        self.analyze_impl_function(function, ctx);
                    })
            }
        )
    }

    fn analyze_self(
        &self,
        self_expression: &SelfExpression,
        context: &mut FirstSemanticsPassContext
    )
    {
        // todo: what if self is captured within nested closures?
        if context.flow_control.current_block_type != BlockType::Method {
            panic!("{} outside of method", self_expression.token);
        }
    }
}