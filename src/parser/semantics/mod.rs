use std::rc::Rc;
use crate::compiler::c_transpiler::CTranspilerContext;
use crate::parser::errors::ParserError;
use crate::parser::parser::{Parser, ParserResult};
use crate::parser::semantics::flow_control::FlowControlContext;
use crate::parser::semantics::name_resolving::NameResolvingContext;
use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::syntax::ast::Statement;

pub mod scopes;
pub mod flow_control;
pub mod constraints;
pub mod traits;
pub mod name_resolving;


#[derive(Debug, Clone, Default)]
pub struct FirstSemanticsPassContext {
    pub flow_control: FlowControlContext,
    pub name_resolving: NameResolvingContext,
    pub transpile: CTranspilerContext,
}

impl AstContext for FirstSemanticsPassContext { /* type Output = (); */}


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
