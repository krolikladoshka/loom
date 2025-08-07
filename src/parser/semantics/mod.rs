use std::collections::HashMap;
use crate::parser::errors::ParserError;
use crate::parser::parser::ParserResult;
use crate::parser::semantics::flow_control::FlowControlContext;
use crate::parser::semantics::name_scoping::NameScopingContext;
use crate::parser::semantics::traits::{AstContext, ContextMapper, Semantics};
use crate::syntax::ast::{current_id, Ast, AstNodeIndex, Context, Statement};
// use crate::typing::type_validation::TypeValidationContext;
use std::rc::Rc;
use crate::compiler::c_transpiler::CTranspilerContext;
use crate::parser::semantics::flatten_tree::ParserContext;
use crate::typing::type_validation::TypeValidationContext;

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

impl ContextMapper<ParserContext> for FirstSemanticsPassContext {
    fn map(from: ParserContext) -> Self {
        Self::from_parser_context(from)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SecondSemanticsPassContext {
    // pub first_pass: FirstSemanticsPassContext,
    pub parser: ParserContext,
    pub flow_control: FlowControlContext,
    pub name_scoping: NameScopingContext,
    pub type_validation: TypeValidationContext,
}

impl SecondSemanticsPassContext {
    pub fn from_first_pass(
        first_pass: FirstSemanticsPassContext,
    ) -> Self {
        Self {
            parser: first_pass.parser,
            flow_control: first_pass.flow_control,
            name_scoping: first_pass.name_scoping,
            type_validation: TypeValidationContext::new(),
        }
    }
}

impl AstContext for SecondSemanticsPassContext { }

impl ContextMapper<FirstSemanticsPassContext> for SecondSemanticsPassContext {
    fn map(from: FirstSemanticsPassContext) -> Self {
        Self::from_first_pass(from)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TranspilerPassContext {
    pub parser: ParserContext,
    pub flow_control: FlowControlContext,
    pub name_scoping: NameScopingContext,
    pub type_validation: TypeValidationContext, 
    pub transpile: CTranspilerContext,
}

impl AstContext for TranspilerPassContext {}

impl ContextMapper<SecondSemanticsPassContext> for TranspilerPassContext {
    fn map(from: SecondSemanticsPassContext) -> Self {
        Self {
            parser: from.parser,
            flow_control: from.flow_control,
            name_scoping: from.name_scoping,
            type_validation: from.type_validation,
            transpile: CTranspilerContext::default(),
        }
    }
}


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
    
    pub fn with_semantics<T: Semantics<SharedContext> + Default + 'static>(
        mut self
    ) -> Self {
        self.analyzers.push(Rc::new(T::default()));
        
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
