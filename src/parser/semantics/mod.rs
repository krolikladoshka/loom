use std::marker::PhantomData;
use std::rc::Rc;
use crate::parser::errors::ParserError;
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
    analyzers: Vec<Rc<dyn Semantics<SharedContext>>>,
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

    pub fn analyze_next(&mut self, statement: &Statement, context: &mut SharedContext) {
        for analyzer in self.analyzers.iter() {
            analyzer.visit_next(statement, context);
        }
    }

    pub fn analyze(&mut self) -> SharedContext {
        let mut context = SharedContext::default();

        loop {
            let parse_result = self.parser.parse_next();
            match parse_result {
                Ok(statement) => {
                    self.analyze_next(&statement, &mut context);
                },
                Err(ParserError::Eof) => break,
                Err(e) => {
                    panic!("{}", e);
                }
            }

        }
        
        context
    }
}
