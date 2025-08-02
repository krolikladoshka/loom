use std::env;
use std::fs::read_to_string;
use std::path::Path;
use crate::parser::parser::Parser;
use crate::syntax::lexer::{Lexer, Token};
use crate::syntax::tokens::TokenType;

pub fn create_test_parser(filepath: &'static str) -> Parser {
    println!("cwd: {}", env::current_dir().unwrap().display());
    let file = read_to_string(Path::new(filepath)).unwrap();
    let mut lexer = Lexer::new(file);
    let tokens = lexer.lex();

    let parser = Parser::new(tokens);

    parser
}

pub fn ttoken(
    token_type: TokenType,
    lexeme: &'static str,
    literal: &'static str,
) -> Token {
    Token::new(
        token_type,
        String::from(lexeme),
        String::from(literal),
        0,
        0,
    )
}

pub fn tidentifier(lexeme: &'static str) -> Token {
    Token::new(
        TokenType::Identifier,
        String::from(lexeme),
        String::from(lexeme),
        0,
        0,
    )
}

pub fn tsemicolon() -> Token {
    Token::new(
        TokenType::Semicolon,
        String::from(";"),
        String::default(),
        0,
        0,
    )
}

pub fn primitive_type_as_str(token_type: TokenType) -> &'static str {
    match token_type {
        TokenType::I8 => "i8",
        TokenType::U8 => "u8",
        TokenType::I16 => "i16",
        TokenType::U16 => "u16",
        TokenType::I32 => "i32",
        TokenType::U32 => "u32",
        TokenType::I64 => "i64",
        TokenType::U64 => "u64",
        TokenType::F32 => "f32",
        TokenType::F64 => "f64",
        TokenType::Bool => "bool",
        TokenType::Char => "char",
        TokenType::V2 => "v2",
        TokenType::V3 => "v3",
        TokenType::V4 => "v4",
        _ => panic!("can only stringify primitive types")
    }
}

pub fn primitive_op_as_str(token_type: TokenType) -> &'static str {
    match token_type {
        TokenType::Star => "*",
        TokenType::Plus => "+",
        TokenType::Minus => "-",
        TokenType::Slash => "/",
        TokenType::Percent => "%",
        _ => panic!("can only stringify primitive operators")
    }
}

#[macro_export]
macro_rules! ttypean {
    ($lit:literal $(, $is_mut:expr)?) => {
        TypeAnnotation::new(
            TypeKind::Simple(
                Type { name: ttoken(TokenType::Identifier, $lit, $lit) }
            ),
            // name: ttoken(TokenType::Identifier, $lit, $lit),
            false $(|| $is_mut )?,
        )
    };
    ($t:path $(, $is_mut:expr)?) => {
        TypeAnnotation::new( 
            TypeKind::Simple(
                Type { name: ttoken($t, primitive_type_as_str($t), "") }
            ),
            // name: ttoken($t, primitive_type_as_str($t), ""),
            false $(|| $is_mut)?,
        )
    };
}

#[macro_export]
macro_rules! ttypedecl {
    ($name:expr, $lit:literal $(, $is_mut:expr)?) => {
        TypedDeclaration::new( 
            tidentifier($name),
            ttypean!($lit $(, $is_mut)?),
        )
    };
    ($name:expr, $t:path $(, $is_mut:expr)?) => {
        TypedDeclaration::new(
            tidentifier($name),
            ttypean!($t $(, $is_mut)?),
        )
    };
}

#[macro_export]
macro_rules! ttypeanptr {
     ($lit:literal, $points_to_mut:expr $(, $is_mut:expr)?) => {
        TypeAnnotation::new(
            TypeKind::Pointer(
                PointerAnnotation {
                    inner_type: Box::new(TypeKind::Simple(Type {
                        name: ttoken(TokenType::Identifier, $lit, $lit),
                    })),
                    points_to_mut: $points_to_mut
                },
            ),
            false $(|| $is_mut )?,
        )
    };
    ($t:path, $points_to_mut:expr $(, $is_mut:expr)?) => {
        TypeAnnotation::new( 
            TypeKind::Pointer(
                PointerAnnotation {
                    inner_type: Box::new(TypeKind::Simple(Type {
                        name: ttoken($t, primitive_type_as_str($t), ""),
                    })),
                    points_to_mut: $points_to_mut
                }
            ),
            false $(|| $is_mut)?,
        )
    };
}

#[macro_export]
macro_rules! ttypedeclptr {
    ($name:expr, $lit:literal, $points_to_mut:expr $(, $is_mut:expr)?) => {
        TypedDeclaration::new( 
            tidentifier($name),
            ttypeanptr!(
                $lit, $points_to_mut $(, $is_mut)?
            )
        )
    };
    ($name:expr, $t:path, $points_to_mut:expr $(, $is_mut:expr)?) => {
        TypedDeclaration::new(
            tidentifier($name),
            ttypeanptr!(
                $t, $points_to_mut $(, $is_mut)?
            )
        )
    };
}

#[macro_export]
macro_rules! tbinary {
    ($left:literal, $left_t:path, $op:path, $right:literal, $right_t:path) => {
        Expression::new_binary(
            Expression::Literal($left_t {
                token: ttoken($left_t, "", ""),
                value: $left
            }),
            ttoken($op, primitive_op_as_str($op), ""),
            Expression::Literal($right_t {
                token: ttoken($right_t, "", ""),
                value: $right
            }),
        )
    };
    ($left:literal, $op:path, $right:literal) => {
       Expression::new_binary(
            Expression::new_identifier(
                tidentifier($left)
            ),
            ttoken($op, primitive_op_as_str($op), ""),
            Expression::new_identifier(
                tidentifier($right),
            )
        )
    };
}