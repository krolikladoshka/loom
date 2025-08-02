use std::collections::HashMap;
use crate::parser::errors::ParserError;
use crate::parser::parser::ParserResult;
use crate::parser::semantics::flow_control::FlowControlContext;
use crate::parser::semantics::name_scoping::NameScopingContext;
use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::syntax::ast::{current_id, Ast, AstNodeIndex, Statement};
// use crate::typing::type_validation::TypeValidationContext;
use std::rc::Rc;
use crate::parser::semantics::flatten_tree::ParserContext;

pub mod scopes;
pub mod flow_control;
pub mod constraints;
pub mod traits;
pub mod name_scoping;
pub mod scope_resolving;
pub mod flatten_tree;

#[derive(Debug, Clone, Default)]
pub struct FirstSemanticsPassContext {
    pub parser: ParserContext,
    pub flow_control: FlowControlContext,
    pub name_scoping: NameScopingContext,
}

impl AstContext for FirstSemanticsPassContext { /* type Output = (); */}

impl FirstSemanticsPassContext {
    pub fn from_parser_context(
        parser_context: ParserContext,
    ) -> Self {
        Self {
            parser: parser_context,
            flow_control: FlowControlContext::default(),
            name_scoping: NameScopingContext::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SecondSemanticsPassContext {
    pub first_pass: FirstSemanticsPassContext,
    // pub type_validation: TypeValidationContext,
}

impl SecondSemanticsPassContext {
    pub fn from_first_pass(
        first_pass: FirstSemanticsPassContext,
    ) -> Self {
        Self {
            first_pass,
            // type_validation: TypeValidationContext::default(),
        }
    }
}

impl AstContext for SecondSemanticsPassContext { }

pub struct SemanticsAnalyzer<'a, SharedContext>
where
    SharedContext: AstContext,
{
    parser_results: &'a [ParserResult<Statement>],
    analyzers: Vec<Rc<dyn Semantics<SharedContext>>>,
}

impl<'a, SharedContext> SemanticsAnalyzer<'a, SharedContext>
where
    SharedContext: AstContext
{
    pub fn new(parser_results: &'a [ParserResult<Statement>])
               -> SemanticsAnalyzer<'a, SharedContext>
    {
        SemanticsAnalyzer::<SharedContext> {
            parser_results,
            analyzers: vec![], // _marker: Default::default()
        }
    }

    pub fn with<T: Semantics<SharedContext> + 'static>(
        mut self,
        analyzer: T
    ) -> Self {
        self.analyzers.push(Rc::new(analyzer));

        self
    }

    pub fn add<T: Semantics<SharedContext> + 'static>(
        &mut self,
        semantics: T,
    )
    {
        self.analyzers.push(Rc::new(semantics));
    }

    pub fn analyze_next(&self, statement: &Statement, context: &mut SharedContext) {
        for analyzer in self.analyzers.iter() {
            analyzer.visit_next(statement, context);
        }
    }

    pub fn analyze_with_context(&self, mut context: SharedContext) -> SharedContext {
        for parser_result in self.parser_results.iter() {
            match parser_result {
                Ok(statement) =>
                    self.analyze_next(statement, &mut context),
                Err(ParserError::Eof) => break,
                Err(e) => panic!(
                    "Parser error during semantics analyzing: {}", e
                )
            }
        }

        context
    }
} 
