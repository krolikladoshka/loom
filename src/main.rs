use std::fs::read_to_string;
use std::path::Path;
use crate::parser::parser::Parser;
use crate::syntax::lexer::Lexer;

pub mod compiler;
pub mod parser;
pub mod typing;
pub mod syntax;
pub mod utils;



fn main() {
    let program = read_to_string(
        Path::new("./resources/structexprfntest.lr")
    ).unwrap();
    
    let mut lexer = Lexer::new(program);
    
    let tokens = lexer.lex();
    tokens.iter().for_each(|t| println!("{:?}", t));
    let mut parser = Parser::new(tokens);
    let (ast, error) = parser.parse();
    
    match error {
        Some(e) => println!("{:?}", e),
        None => {}
    };
    
    println!("{:?}", ast)
}
