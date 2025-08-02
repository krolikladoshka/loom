pub mod compiler;
pub mod parser;
pub mod typing;
pub mod syntax;
pub mod utils;

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

    let flow_control_semantics = FlowControlSemantics {};
    let name_scoping_semantics = NameScopingSemantics {};
    let first_pass = SemanticsAnalyzer::new(
        &parser_results
    )
        .with(flow_control_semantics)
        .with(name_scoping_semantics);
    // let second_pass = SemanticsAnalyzer::new(
    //     &parser_results
    // )
    //     .with(name_table_semantics);
        // .with(type_validation_semantics);
    

    // let scope_resolving_semantics = NameResolvingSemantics {};
    // let typing_validation_semantics = TypingValidationSemantics {};
    // let second_pass = SemanticsAnalyzer::new(
    //     &parser_results
    // )
    //     .with(scope_resolving_semantics)
        // .with(typing_validation_semantics);

    // let c_transpiler_semantics = CTranspilerSemantics {};
    // let third_pass = SemanticsAnalyzer::new(
    //     &parser_results
    // )
    //     .with(c_transpiler_semantics);
    
    let ast_context = FirstSemanticsPassContext::default()
        .then_analyze_by(first_pass, |x| x);
        // .then_analyze_by(second_pass, SecondSemanticsPassContext::from_first_pass);
        // .then_analyze_by(
        //     second_pass,
        //     |mut first_context| SecondSemanticsPassContext {
        //         first_semantics_pass_context: first_context,
        //         new_data: vec![]
        //     }
        // )
        // .then_analyze_by(
        //     third_pass,
        //     |mut second_context| ThirdSemanticsPassContext {
        //         first_semantics_pass_context: &mut second_context.first_semantics_pass_context,
        //         second_semantics_pass_context: second_context,
        //         transpile: Default::default(),
        //     }
        // );

    println!("{:?}", ast_context);
    // let ast_context = analyzer.analyze();

    // println!("{:?}", ast_context);

    // println!("{}", ast_context.transpile.result);
    // let mut file = File::create("./loom_runtime/main.c").unwrap();
    // file.write_all(ast_context.transpile.result.as_bytes()).unwrap();
    // file.flush().unwrap();
    // 
    // println!("{}", ast_context.transpile.transpile_results.len());

    // let (ast, error) = parser.parse();
    // 
    // match error {
    //     Some(e) => for q in e {
    //         println!("{}", q);
    //     }
    //     None => {}
    // };
    // 
    // println!("{}", ast.print_tree(0))
    enum AstNode {
        Expression(Expression),
        Statement(Statement),
    }
    // let mut arena = Vec::with_capacity(current_id());
    // arena.push(RefCell::new(parser.parse_next()));
    // let mut transpiler = CTranspiler::new(&mut parser);
    //
    // let result = transpiler.transpile();
    
    // println!("{}", result);
}
