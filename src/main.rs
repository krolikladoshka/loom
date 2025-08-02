pub mod compiler;
pub mod parser;
pub mod typing;
pub mod syntax;
pub mod utils;

use crate::parser::semantics::flatten_tree::FlattenTree;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
// use crate::compiler::c_transpiler::{CTranspilerContext, CTranspilerSemantics};
// use crate::compiler::c_transpiler::CTranspiler;
use crate::parser::parser::Parser;
use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::parser::semantics::{FirstSemanticsPassContext, SecondSemanticsPassContext, SemanticsAnalyzer};
use crate::syntax::ast::{Expression, Statement};
use crate::syntax::lexer::Lexer;
use crate::syntax::traits::TreePrint;
use crate::parser::semantics::flow_control::{FlowControlContext, FlowControlSemantics};
use crate::parser::semantics::name_scoping::{NameScopingContext, NameScopingSemantics};

use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use crate::compiler::c_transpiler::CTranspilerSemantics;
use crate::parser::semantics::flatten_tree::ParserContext;
use crate::parser::semantics::scope_resolving::ScopeResolvingSemantics;

pub struct Test {
    pub field: i32,
}

fn test() {
    let t = Test { field: 1};
}


// pub struct ThirdSemanticsPassContext<'a> {
//     pub first_semantics_pass_context: &'a mut FirstSemanticsPassContext,
//     pub second_semantics_pass_context: SecondSemanticsPassContext,
//     pub transpile: CTranspilerContext
// }
// impl AstContext for SecondSemanticsPassContext {
//     // type Output = ();
// }
// impl AstContext for ThirdSemanticsPassContext<'_> {
//     // type Output = ();
// }
//
// struct SecondSemanticsPass;
// impl Semantics<SecondSemanticsPassContext<'_>> for SecondSemanticsPass {}



// pub struct TypingValidationSemantics;
// impl Semantics<SecondSemanticsPassContext<'_>> for TypingValidationSemantics {}
//
// impl Semantics<ThirdSemanticsPassContext<'_>> for CTranspilerSemantics {}


trait Referenced<'a> {
    fn on_ref(&self, q: &'a i32) -> i32 {
        q.clone()
    }
}

struct Testq {
    pub field: i32,
}

impl Referenced<'_> for Testq {
    fn on_ref(&self, q: &i32) -> i32 {
        q.clone() + self.field
    }
}

fn q() {
    Testq { field: 42 }.on_ref(&3);
}

fn main() {
    let program = read_to_string(
        Path::new("./resources/example.rs")
    ).unwrap();
    
    let mut lexer = Lexer::new(program);
    
    let tokens = lexer.lex();
    // tokens.iter().for_each(|t| println!("{:?}", t));
    let mut parser = Parser::new(tokens);
   
    /*
     0. Ast Construction from parsing
    1. FlowControlValidation + NameTableConstruction
    2. TypingValidation + ScopeResolving
    3. Transpilation
     */
    let (_, errors, parser_results ) = parser.parse();

    let flatten_tree_constructor = SemanticsAnalyzer::new(
        &parser_results
    )
        .with_semantics::<FlattenTree>();
    let first_pass = SemanticsAnalyzer::new(
        &parser_results
    )
        .with_semantics::<FlowControlSemantics>()
        .with_semantics::<NameScopingSemantics>();
    let second_pass = SemanticsAnalyzer::new(
        &parser_results
    )
        .with_semantics::<ScopeResolvingSemantics>();
        // .with_semantics::<TypingSemantics>();

    let ast_context = ParserContext::default()
        .analyze_by(flatten_tree_constructor)
        .then_analyze_by(
            first_pass
        )
        .then_analyze_by(
            second_pass
        ).then_analyze_by(
            SemanticsAnalyzer::new(&parser_results)
                .with_semantics::<CTranspilerSemantics>()
        );

    println!("{:?}", ast_context);
    println!("{}", ast_context.transpile.result);
    let mut file = File::create("./loom_runtime/transpiled.c").unwrap();
    file.write_all(ast_context.transpile.result.as_bytes()).unwrap();
    file.flush().unwrap();
}
