pub mod compiler;
pub mod parser;
pub mod typing;
pub mod syntax;
pub mod utils;

use std::cell::RefCell;
use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::ptr::write;
use crate::compiler::c_transpiler::CTranspilerSemantics;
// use crate::compiler::c_transpiler::CTranspiler;
use crate::parser::parser::Parser;
use crate::parser::semantics::flow_control::FlowControlSemantics;
use crate::parser::semantics::SemanticsAnalyzer;
use crate::syntax::ast::{current_id, Expression, Statement};
use crate::syntax::lexer::Lexer;
use crate::syntax::traits::TreePrint;

pub struct Test {
    pub field: i32,
}

fn test() {
    let t = Test { field: 1};
}
fn main() {
    let program = read_to_string(
        Path::new("./resources/example.rs")
    ).unwrap();
    
    let mut lexer = Lexer::new(program);
    
    let tokens = lexer.lex();
    // tokens.iter().for_each(|t| println!("{:?}", t));
    let mut parser = Parser::new(tokens);
    
    let flow_control_semantics: FlowControlSemantics = FlowControlSemantics {};
    let c_transpiler_semantics = CTranspilerSemantics {};
    let mut analyzer = SemanticsAnalyzer::new(&mut parser);
    analyzer.add(flow_control_semantics);
    analyzer.add(c_transpiler_semantics);

    let ast_context = analyzer.analyze();

    // println!("{:?}", ast_context);

    // println!("{}", ast_context.transpile.result);
    let mut file = File::create("./loom_runtime/main.c").unwrap();
    file.write_all(ast_context.transpile.result.as_bytes()).unwrap();
    file.flush().unwrap();

    println!("{}", ast_context.transpile.transpile_results.len());

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
