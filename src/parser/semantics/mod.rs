use std::marker::PhantomData;
use crate::parser::parser::Parser;
use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::syntax::ast::Statement;

pub mod scopes;
pub mod flow_control;
pub mod constraints;
pub mod traits;


pub struct SemanticsAnalyzer<'a, Ast, SharedContext>
where
    SharedContext: AstContext,
{
    parser: &'a mut Parser,
    analyzers: Vec<Box<dyn Semantics<SharedContext>>>,
    _marker: PhantomData<Ast>,
}


impl<'a, Ast, SharedContext> SemanticsAnalyzer<'a, Ast, SharedContext>
where
    SharedContext: AstContext<Output = Ast>
{
    pub fn new(parser: &'a mut Parser) -> SemanticsAnalyzer<Ast, SharedContext> {
        SemanticsAnalyzer::<SharedContext::Output, SharedContext> {
            parser, analyzers: vec![], _marker: Default::default()
        }
    }

    pub fn with<T: Semantics<SharedContext> + 'static>(
        &'a mut self,
        analyzer: T
    ) -> &'a mut Self {
        self.analyzers.push(Box::new(analyzer));

        self
    }

    pub fn analyze_next(&mut self, statement: &Statement, context: &mut SharedContext) {
        for analyzer in self.analyzers.iter() { ;
            analyzer.analyze_statement(statement, context);
        }
    }

    pub fn analyze(&mut self) {
        let mut context = SharedContext::default();

        while let Ok(statement) = self.parser.parse_next() {
            self.analyze_next(&statement, &mut context);
        }
    }
}
