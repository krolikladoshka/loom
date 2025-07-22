use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::syntax::ast::WhileStatement;
use std::cell::Cell;

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy, Hash)]
enum BlockType {
    #[default]
    TopLevel,
    Function,
    Loop,
    While,
    For,
}

#[derive(Debug, Clone, Default)]
struct FirstSemanticsPassContext {
    // pub resolved_variables: HashMap<String, Identifier>,
    // pub scopes: Vec<Scope>,
    // pub transformed_ast: HashMap<usize, TransformedAstNode>

    pub previous_block_type: BlockType,
    pub current_block_type: BlockType,
}

impl AstContext for FirstSemanticsPassContext { type Output = (); }

impl FirstSemanticsPassContext {
    pub fn set_block_type(&mut self, block_type: BlockType) {
        self.previous_block_type = self.current_block_type;
        self.current_block_type = block_type;
    }

    pub fn restore_block_type(&mut self) {
        self.current_block_type = self.previous_block_type;
    }

    pub fn scope_block_type<F>(&mut self, block_type: BlockType, block: F)
    where
        F: FnOnce(&mut Self),
    {
        let stored_block_type = self.current_block_type;
        self.current_block_type = block_type;
        block(self);
        self.current_block_type = stored_block_type;
    }

}
pub struct FlowControlSemantics {
    test: Cell<bool>
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
        // context.scope_block_type(
        //     BlockType::While,
        //     |ctx| {
        //         self.analyze_expression(&while_statement.condition, ctx);
        //         self.analyze_all_statements(&while_statement.body, ctx);
        //     }
        // )
    }
}