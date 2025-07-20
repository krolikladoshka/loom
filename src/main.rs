pub mod compiler;
pub mod parser;
pub mod typing;
pub mod syntax;
pub mod utils;


use std::fs::read_to_string;
use std::path::Path;
use crate::compiler::c_transpiler::CTranspiler;
use crate::parser::parser::Parser;
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
    tokens.iter().for_each(|t| println!("{:?}", t));
    let mut parser = Parser::new(tokens);
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
    
    let mut transpiler = CTranspiler::new(&mut parser); 
    
    let result = transpiler.transpile();
    
    println!("{}", result);
}
