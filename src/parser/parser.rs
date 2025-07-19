use std::env;
use std::fmt::Display;
use std::fs::read_to_string;
use std::path::Path;
use std::str::FromStr;
use crate::syntax::ast::{Literal, Expression, Function, ImplFunction, Method, Statement, TypeAnnotation, TypeKind, TypedDeclaration, Type, PointerAnnotation, Identifier};
use crate::syntax::lexer::{Lexer, Token};
use crate::syntax::tokens::TokenType;


#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub position: usize
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{} at p{}:l{}:c{}",
            self.message, self.position,
            self.line, self.column
        )
    }
}

#[derive(Debug, Clone)]
pub struct UnexpectedTokenError {
    pub found: Token,
    pub expected: TokenType,
    pub message: &'static str,
}

impl Display for UnexpectedTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "unexpected token: expected {:?} [{}] but got {}",
            self.expected, self.message, self.found,
        )
    }
}

#[derive(Debug, Clone)]
pub struct UnexpectedEofError {
    pub expected: TokenType,
    pub message: &'static str,
}

impl Display for UnexpectedEofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "unexpected end of stream: expected {:?} [{}]",
            self.expected, self.message
        )
    }
}

#[derive(Debug, Clone)]
pub struct TypeAnnotationRequiredError {
    pub identifier: Token,
    pub message: &'static str,
}

impl Display for TypeAnnotationRequiredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "expected type annotation for {} [{}]",
            self.identifier, self.message
        )
    }
}

#[derive(Debug, Clone)]
pub struct InitializerRequiredError {
    pub identifier: Token,
    pub message: &'static str,
}

impl Display for InitializerRequiredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "expected initializer for {} [{}]",
            self.identifier, self.message
        )
    }
}

#[derive(Debug, Clone)]
pub struct LiteralParseError {
    pub token: Token,
    pub to: &'static str
}

impl Display for LiteralParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "couldn't parse literal from token {} into {}",
            self.token, self.to
        )
    }
}

#[derive(Debug, Clone)]
pub enum ParserError {
    EmptyStream,
    UnrecognizedToken(Token),
    UnexpectedToken(UnexpectedTokenError),
    UnexpectedEof(UnexpectedEofError),
    UnexpectedTopLevelStatement(Token),
    UnexpectedStatement(Token),
    TypeAnnotationRequired(TypeAnnotationRequiredError),
    InitializerRequired(InitializerRequiredError),
    LiteralParseError(LiteralParseError),
    UnexpectedPrimaryExpression(Token),
    UnterminatedArraySlice(Token),
    ParseError(ParseError),
    CompoundError(Vec<ParserError>)
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::EmptyStream => write!(f, "Empty stream"),
            Self::UnrecognizedToken(token) => write!(f, "Unrecognized token: {}", token),
            Self::UnexpectedToken(unexpected_token_error) =>
                write!(f, "{}", unexpected_token_error),
            Self::UnexpectedEof(unexpected_eof_error) =>
                write!(f, "{}", unexpected_eof_error),
            Self::UnexpectedTopLevelStatement(utls) =>
                write!(f, "{}", utls),
            Self::UnexpectedStatement(us) =>
                write!(f, "Unexpected statement {}", us),
            Self::TypeAnnotationRequired(t) =>
                write!(f, "{}", t),
            Self::InitializerRequired(t) =>
                write!(f, "{}", t),
            Self::LiteralParseError(err) =>
                write!(f, "{}", err),
            Self::UnexpectedPrimaryExpression(t) =>
                write!(f, "unexpected primary expression token {}", t),
            Self::UnterminatedArraySlice(t) =>
                write!(f, "unterminated array slice for {}", t),
            Self::ParseError(t) => write!(f, "Uncategorized error: {}", t),
            Self::CompoundError(errors) => {
                let mut messages = vec![];
                for error in errors {
                    messages.push(format!("1:\n\t{}", error));
                }

                messages.join("\n").fmt(f)
            }
        }
    }
}


impl ParserError {
    pub fn unexpected_token(found: Token, expected: TokenType, message: &'static str) -> Self {
        Self::UnexpectedToken(UnexpectedTokenError { found, expected, message})
    }

    pub fn unexpected_eof(expected: TokenType, message: &'static str) -> Self {
        Self::UnexpectedEof(UnexpectedEofError { expected, message })
    }

    pub fn error(message: String, line: usize, column: usize, position: usize) -> Self {
        Self::ParseError(ParseError { message, line, column, position })
    }

    pub fn type_annotation_required(identifier: Token, message: &'static str) -> Self {
        Self::TypeAnnotationRequired(TypeAnnotationRequiredError { identifier, message })
    }

    pub fn initializer_required(identifier: Token, message: &'static str) -> Self {
        Self::InitializerRequired(InitializerRequiredError { identifier, message })
    }

    pub fn literal_parse_error(token: Token, to: &'static str) -> Self {
        Self::LiteralParseError(LiteralParseError {
            token,
            to,
        })
    }
}


type ParserResult<T> = Result<T, ParserError>;


pub struct Parser {
    tokens: Vec<Token>,
    start_position: usize,
    current_position: usize,
    panic_on_error: bool,
}


impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            start_position: 0,
            current_position: 0,
            panic_on_error: false,
        }
    }

    #[inline(always)]
    fn wrap_error(&mut self, error: ParserError) -> ParserError {
        if self.panic_on_error {
            panic!("{}", error);
        }
        error
    }

    #[inline(always)]
    fn is_at_end(&self) -> bool {
        if self.current_position < self.tokens.len() {
            return self.tokens[self.current_position].token_type == TokenType::EOF;
        }
        
        self.current_position >= self.tokens.len()
    }

    fn advance_skipping_comments(&mut self) -> Option<Token> {
        loop {
            if self.is_at_end() {
                return None;
            }

            let token = self.tokens.get(self.current_position)?;

            match token.token_type {
                TokenType::SingleLineComment | TokenType::MultilineCommentStart |
                TokenType::MultilineCommentEnd => {
                    self.current_position += 1;
                },
                _ => return Some(token.clone())
            }
        }
    }
    
    fn advance_and_skip_next_comments(&mut self) -> Option<Token> {
        self.advance()?;
        self.advance_skipping_comments()
    }

    #[inline(always)]
    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        // self.peek_skipping_comments()
        self.tokens.get(self.current_position + offset)
    }

    #[inline(always)]
    fn peek(&self) -> Option<&Token> {
        self.peek_ahead(0)
    }

    #[inline(always)]
    fn advance(&mut self) -> Option<Token> {
        if self.is_at_end() {
            return None;
        }
        let token = self.peek().cloned();
        if self.current_position < self.tokens.len() {
            self.current_position += 1;
        }

        token
    }

    fn peek_protected(&mut self) -> Result<&Token, ParserError> {
        let token = self.advance_skipping_comments();
        self.peek().ok_or(ParserError::EmptyStream)
    }

    fn require(
        &mut self, token_type: TokenType, additional_message: &'static str
    ) -> Result<Token, ParserError> {
        let token = match self.advance() {
            Some(token) => token,
            None => return Err(
                self.wrap_error(ParserError::unexpected_eof(token_type, additional_message))
            )
        };

        if token.token_type != token_type {
            return Err(self.wrap_error(
                ParserError::unexpected_token(
                    token.clone(), token_type, additional_message
                )
            ));
        }

        Ok(token)
    }

    fn require_type_identifier(&mut self) -> ParserResult<Token> {
        const TOKEN_TYPES:  &'static [TokenType] = &[
            TokenType::Identifier,
            TokenType::I8, TokenType::I16,
            TokenType::I32, TokenType::I64,
            TokenType::U8, TokenType::U16,
            TokenType::U32, TokenType::U64,
            TokenType::F32, TokenType::F64,
            TokenType::Bool,
        ];

        let Some(token) = self.peek().cloned() else {
            return Err(self.wrap_error(ParserError::unexpected_eof(
                TokenType::Identifier,
                "expected type identifier",
            )));
        };

        if TOKEN_TYPES.contains(&token.token_type) {
            self.advance();
            Ok(token.clone())
        } else {
            Err(self.wrap_error(ParserError::unexpected_token(
                token.clone(),
                TokenType::Identifier,
                "expected type identifier",
            )))
        }
    }


    fn match_tokens(
        &mut self, token_types: &[TokenType], msg: &'static str
    ) -> ParserResult<Option<Token>> {
        self.advance_skipping_comments();

        let Some(token) = self.peek().cloned() else {
            return Err(self.wrap_error(ParserError::unexpected_eof(
                token_types[0].clone(),
                msg
            )));
        };

        if token_types.contains(&token.token_type) {
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    fn parse_panic(&mut self) -> Vec<Statement> {
        self.panic_on_error = true;

        let (result, _) = self.parse();

        result
    }

    pub fn parse(&mut self) -> (Vec<Statement>, Option<ParserError>) {
        let mut ast = vec![];
        let mut errors = vec![];

        loop {
            if self.is_at_end() {
                break;
            }

            self.advance_skipping_comments();

            let parse_result = self.top_level_statement();
            match parse_result {
                Ok(statement) => ast.push(statement),
                Err(err) => {
                    errors.push(err);
                    self.advance_until_next_statement();
                    self.empty_statement();
                },
            }
        }

        let error = if errors.is_empty() {
            None
        } else {
            Some(ParserError::CompoundError(errors))
        };

        (ast, error)
    }

    fn advance_until_next_statement(&mut self) {
        while !self.is_at_end() {
            let token = self.peek().unwrap();

            if token.token_type == TokenType::Semicolon {
                return;
            }
            self.advance();
        }
    }

    //// statements
    fn empty_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(TokenType::Semicolon, "Expected semicolon")?;

        Ok(Statement::EmptyStatement {
            semicolon_token: token
        })
    }

    pub fn top_level_statement(&mut self) -> ParserResult<Statement> {
        let token = self.peek_protected()?.clone();

        match token.token_type {
            TokenType::SingleLineComment | TokenType::MultilineCommentEnd |
            TokenType::MultilineCommentStart => {
                self.advance_skipping_comments();
                self.top_level_statement()
            },
            TokenType::Semicolon => self.empty_statement(),
            TokenType::Use => self.parse_top_level_use(),
            TokenType::Pub => self.parse_top_level_pub(),
            TokenType::Impl => self.impl_statement(),
            _ => match self.structs_functions_and_variable_declarations() {
                Ok(statement) => Ok(statement),
                Err(ParserError::UnexpectedStatement(token)) => Err(
                    self.wrap_error(ParserError::UnexpectedTopLevelStatement(token))
                ),
                Err(e) => Err(self.wrap_error(e))
            }
        }
    }

    fn structs_functions_and_variable_declarations(
        &mut self
    ) -> ParserResult<Statement> {
        self.advance_skipping_comments();
        let token = self.peek_protected()?.clone();

        match token.token_type {
            TokenType::Struct => self.struct_statement(),
            TokenType::EnumStruct => todo!("Enum struct"), // self.enum_struct_statement(),
            TokenType::UnionStruct => todo!("Union struct"), // self.union_struct_statement(),
            _ => self.variable_declarations_and_functions(),

        }
    }

    fn impl_pub_statement(&mut self) -> ParserResult<Statement> {
        let pub_token = match self.peek().cloned() {
            Some(token) => if token.token_type == TokenType::Pub {
                self.advance();

                Some(token)
            } else {
                None
            },
            None => {
                return Err(self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Pub,
                    "expected impl block statement",
                )));
            }
        };

        // TODO: set pub modifier
        let statement = self.variable_declarations_and_functions()?;

        Ok(statement)
    }

    fn variable_declarations_and_functions(&mut self) -> ParserResult<Statement> {
        self.advance_skipping_comments();
        let token = self.peek_protected()?.clone();

        match token.token_type {
            TokenType::Static => self.static_variable_statement(),
            TokenType::Const => self.const_variable_statement(),
            TokenType::Fn => self.function_statement(),
            _ => Err(self.wrap_error(ParserError::UnexpectedStatement(token))),
        }
    }

    fn parse_top_level_use(&mut self) -> ParserResult<Statement> {
        todo!()
        // let use_m/**/self.require(TokenType::Use, "Top level use")?;
    }

    fn parse_top_level_pub(&mut self) -> ParserResult<Statement> {
        let pub_token = self.require(TokenType::Pub, "expected 'pub'")?;

        match self.structs_functions_and_variable_declarations() {
            Ok(statement) => {
                // TODO: another struct with pub modifiers for names?
                // statement.is_pub = true;

                Ok(statement)
            },
            Err(e) => Err(self.wrap_error(e))
        }
    }

    fn statement(&mut self) -> ParserResult<Statement> {
        self.advance_skipping_comments();
        let token = self.peek_protected()?.clone();

        match token.token_type {
            TokenType::Semicolon => self.empty_statement(),
            TokenType::Let => self.let_variable_statement(),
            TokenType::Static => self.static_variable_statement(),
            TokenType::Const => self.const_variable_statement(),
            TokenType::Fn => self.function_statement(),
            TokenType::Struct => self.struct_statement(),
            TokenType::EnumStruct => todo!(), // self.enum_struct_statement(),
            TokenType::UnionStruct => todo!(), // self.union_struct_statement(),
            TokenType::Defer => todo!("defer statement"), // self.defer_statement(),
            TokenType::Use => todo!("use statement"), // self.use_statement(),
            TokenType::While => self.while_statement(),
            TokenType::Break => self.break_statement(),
            TokenType::Continue => self.continue_statement(),
            TokenType::Return => self.return_statement(),
            _ => self.expression_statement()
        }
    }

    fn match_current(
        &mut self, expected: TokenType, msg: &'static str
    )-> ParserResult<Option<Token>> {
        self.advance_skipping_comments();

        let Some(token) = self.peek().cloned() else {
            return Err(self.wrap_error(ParserError::unexpected_eof(
                expected,
                msg
            )));
        };

        if token.token_type == expected {
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    fn simple_type(&mut self) -> ParserResult<Type> {
        self.advance_skipping_comments();

        Ok(Type {
            name: self.require_type_identifier()?
        })
    }

    fn pointer_type(&mut self) -> ParserResult<PointerAnnotation> {
        self.advance_skipping_comments();
        self.require(TokenType::Star, "expected '*'")?;
        let mut points_to_mut = false;

        if self.match_current(
            TokenType::Mut, "expected mut modifier"
        )?.is_some() {
            self.advance();
            self.advance_skipping_comments();

            points_to_mut = true;
        } else if self.match_current(
            TokenType::Const, "expected const modifier"
        )?.is_some() {
            self.advance();
            self.advance_skipping_comments();

            points_to_mut = false
        }

        if let Some(_) = self.match_current(
            TokenType::Star, "Expected '*' or type identifier"
        )? {
            let sub_ptr = self.pointer_type()?;

            Ok(PointerAnnotation {
                inner_type: Box::new(TypeKind::Pointer(sub_ptr)),
                points_to_mut
            })
        } else {
            let simple_type = self.simple_type()?;


            Ok(PointerAnnotation {
                inner_type: Box::new(TypeKind::Simple(simple_type)),
                points_to_mut
            })
        }
    }

    fn type_annotation(&mut self, no_mut: bool) -> ParserResult<TypeAnnotation> {
        let mut is_mut = false;
        if !no_mut {
            let Ok(token) = self.peek_protected() else {
                return Err(self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Mut, "expected mut modifier"
                )));
            };

            if token.token_type == TokenType::Mut {
                self.advance();
                self.advance_skipping_comments();

                is_mut = true;
            };
        }

        let Some(token) = self.peek().cloned() else {
            return Err(self.wrap_error(ParserError::unexpected_eof(
                TokenType::Star,
                "expected '*' or type identifier"
            )));
        };

        if token.token_type == TokenType::Star {
            // self.advance();
            // self.advance_skipping_comments();

            let pointer_type = self.pointer_type()?;

            Ok(TypeAnnotation {
                kind: TypeKind::Pointer(pointer_type),
                is_mut,
            })
        } else {
            let simple_type = self.simple_type()?;

            Ok(TypeAnnotation {
                kind: TypeKind::Simple(simple_type),
                is_mut,
            })
        }
    }

    fn common_variable_declaration<T, Constructor>(
        &mut self,
        no_mut: bool,
        constructor: T
    ) -> ParserResult<Constructor>
    where
        T: FnOnce(
            Token, Option<TypeAnnotation>, Option<Box<Expression>>
        ) -> ParserResult<Constructor>
    {
        let identifier_token = self.require(
            TokenType::Identifier,
            "expected a variable name"
        )?;

        let Some(token) = self.peek() else {
            return Err(self.wrap_error(ParserError::unexpected_eof(
                TokenType::Equals,
                "expected type annotation or assignment"
            )));
        };

        let variable_type =
            if token.token_type == TokenType::Colon {

                self.advance();
                self.advance_skipping_comments();
                Some(self.type_annotation(no_mut)?)
        } else {
            None
        };

        let token = self.peek_protected()?;
        let initializer = if token.token_type == TokenType::Equals {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        constructor(identifier_token, variable_type, initializer)
    }

    fn let_variable_statement(&mut self) -> ParserResult<Statement> {
        let let_token = self.require(
            TokenType::Let,
            "expected 'let'",
        )?;

        let result = self.common_variable_declaration(
            false,
            |identifier, type_annotation, initializer|
                Ok(Statement::LetStatement {
                    token: let_token,
                    name: identifier,
                    variable_type: type_annotation,
                    initializer
                })
        )?;

        self.require(
            TokenType::Semicolon,
            "expected ';' at the end of let statement"
        )?;

        Ok(result)
    }

    fn static_variable_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(
            TokenType::Static,
            "expected 'static'",
        )?;

        let result = self.common_variable_declaration(
            false,
            |identifier, type_annotation, initializer| {
                let annotation = match type_annotation {
                    Some(type_annotation) => type_annotation,
                    _ => return Err(ParserError::type_annotation_required(
                        identifier,
                        "static variables are required to have a type annotation"
                    ))
                };
                let initializer = match initializer {
                    Some(initializer) => initializer,
                    _ => return Err(ParserError::initializer_required(
                        identifier,
                        "static variables must be initialized"
                    ))
                };

                Ok(Statement::StaticStatement {
                    token,
                    name: identifier,
                    variable_type: annotation,
                    initializer
                })
            }
        )?;

        self.require(
            TokenType::Semicolon,
            "expected ';' at the end of static variable statement"
        )?;

        Ok(result)
    }

    fn const_variable_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(
            TokenType::Const,
            "expected 'const'"
        )?;

        let result = self.common_variable_declaration(
            true,
            |identifier, type_annotation, initializer| {
                let annotation = match type_annotation {
                    Some(type_annotation) => type_annotation,
                    _ => return Err(ParserError::type_annotation_required(
                        identifier,
                        "const variables are required to have a type annotation"
                    ))
                };
                let initializer = match initializer {
                    Some(initializer) => initializer,
                    _ => return Err(ParserError::initializer_required(
                        identifier,
                        "const variables must be initialized"
                    ))
                };

                Ok(Statement::StaticStatement {
                    token,
                    name: identifier,
                    variable_type: annotation,
                    initializer
                })
            }
        )?;

        self.require(
            TokenType::Semicolon,
            "expected ';' at the end of const variable statement"
        )?;

        Ok(result)
    }

    fn function_start(&mut self) -> ParserResult<(Token, Token)>{
        let token = self.require(
            TokenType::Fn,
            "expected 'fn'"
        )?;


        let function_name = self.require(
            TokenType::Identifier,
            "expected a function name in fn statement"
        )?;

        self.require(
            TokenType::LeftParenthesis,
            "expected '(' after function name"
        )?;

        Ok((token, function_name))
    }

    fn function_argument(&mut self, token: Token) -> ParserResult<TypedDeclaration> {
        let mut is_mut = false;
        if token.token_type == TokenType::Mut {
            self.advance();
            is_mut = true;
        }

        let argument_name = self.require(
            TokenType::Identifier,
            "expected parameter name in function declaration"
        )?;


        self.require(
            TokenType::Colon,
            "expected ':' after parameter name"
        )?;

        let mut annotation = self.type_annotation(true)?;
        annotation.is_mut = is_mut;

        Ok(TypedDeclaration {
            name: argument_name,
            declared_type: annotation,
        })
    }

    fn function_end(&mut self) -> ParserResult<(Option<TypeAnnotation>, Vec<Statement>)>
    {
        self.require(
            TokenType::RightParenthesis,
            "expected ')' after arguments list"
        )?;

        let type_identifier = match self.peek() {
            Some(token) => if token.token_type == TokenType::Arrow {
                self.advance();

                let type_annotation = self.type_annotation(true)?;
                Some(type_annotation)
            } else {
                None
            },
            None => None
        };

        let body = self.parse_block_of_statements()?;

        Ok((type_identifier, body))
    }

    fn impl_function_statement(&mut self, bound_type: Token) -> ParserResult<Statement> {
        let (token, function_name) = self.function_start()?;

        let mut is_bound_method = false;
        let mut is_mut_self = false;
        let mut arguments = vec![];

        loop {
            let token = match self.peek().cloned() {
                Some(token) => token,
                None => return Err(self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Identifier,
                    "Expected argument or closing parenthesis"
                )))
            };

            if token.token_type == TokenType::RightParenthesis {
                break;
            }

            if arguments.len() < 1 {
                if token.token_type == TokenType::Star {
                    self.advance();
                }
                is_bound_method = true;

                let Some(mut_token) = self.peek().cloned() else {
                    return Err(self.wrap_error(ParserError::unexpected_eof(
                        TokenType::SelfToken,
                        "expected 'self' after * in bound method"
                    )));
                };

                if mut_token.token_type == TokenType::Mut {
                    self.advance();
                    is_mut_self = true;
                }
                self.require(
                    TokenType::SelfToken,
                    "expected 'self' after * in bound method"
                )?;
            } else {
                let typed_declaration = self.function_argument(token)?;
                arguments.push(typed_declaration);
            }
            let token = match self.peek() {
                Some(token) => token,
                None => return Err(self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Comma,
                    "expected comma or closing parenthesis"
                )))
            };

            if token.token_type != TokenType::Comma {
                break;
            }
        }

        self.require(
            TokenType::RightParenthesis,
            "expected ')' after arguments list"
        )?;

        let (type_identifier, body) = self.function_end()?;

        let function = if is_bound_method {
            ImplFunction::Method(Method {
                name: function_name,
                bound_type,
                is_mut_self,
                arguments,
                return_type: type_identifier,
                body
            })
        } else {
            ImplFunction::Function(Function {
                name: function_name,
                arguments,
                return_type: type_identifier,
                body
            })
        };


        Ok(Statement::FnStatement {
            token,
            function
        })
    }

    fn function_statement(&mut self) -> ParserResult<Statement> {
        let (token, function_name) = self.function_start()?;

        let mut arguments = vec![];
        loop {
            let token = match self.peek().cloned() {
                Some(token) => token,
                None => return Err(self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Identifier,
                    "Expected argument or closing parenthesis"
                )))
            };

            if token.token_type == TokenType::RightParenthesis {
                break;
            }

            let typed_declaration = self.function_argument(token)?;
            arguments.push(typed_declaration);

            let token = match self.peek() {
                Some(token) => token,
                None => return Err(self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Comma,
                    "expected comma or closing parenthesis"
                )))
            };

            if token.token_type != TokenType::Comma {
                break;
            }
            self.advance();
        }

        let (type_identifier, body) = self.function_end()?;

        let function = ImplFunction::Function(Function {
            name: function_name,
            arguments,
            return_type: type_identifier,
            body
        });

        Ok(Statement::FnStatement {
            token,
            function
        })
    }

    fn struct_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(
            TokenType::Struct,
            "expected 'struct' keyword"
        )?;

        let name = self.require(
            TokenType::Identifier,
            "expected a struct name in struct declaration"
        )?;

        self.require(
            TokenType::LeftBrace,
            "expected '{' after struct name"
        )?;

        let mut fields = vec![];
        loop {
            let Some(token) = self.advance_skipping_comments() else {
                return Err(self.wrap_error(
                    ParserError::unexpected_eof(
                        TokenType::Identifier,
                        "expected struct definition after '{' after 'struct'"
                    )
                ));
            };
            // let Some(token) = self.peek().cloned() else {
            //     return Err(self.wrap_error(ParserError::unexpected_eof(
            //         TokenType::Identifier,
            //         "expected struct definition after '{' after 'struct'"
            //     )))
            // };

            if token.token_type == TokenType::RightBrace {
                break;
            }

            let field = self.common_variable_declaration(
                true,
                |identifier, type_annotation, initializer: Option<Box<Expression>>| {
                    let Some(annotation) = type_annotation else {
                        return Err(ParserError::type_annotation_required(
                            identifier,
                            "type annotation is required for fields in structs"
                        ));
                    };

                    // TODO: that is temporary forever
                    if initializer.is_some() {
                        return Err(ParserError::unexpected_token(
                            token.clone(),
                            TokenType::Comma,
                            "struct fields can't have initializers"
                        ));
                    };

                    Ok(TypedDeclaration {
                        name: identifier,
                        declared_type: annotation,
                    })
                }
            )?;
            fields.push(field);

            let Some(token) = self.advance_skipping_comments() else {
                return Err(self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Comma,
                    "expected ',' or '}' after struct field definition"
                )));
            };

            if token.token_type == TokenType::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.require(
            TokenType::RightBrace,
            "expected '}' after struct definition"
        )?;

        // TODO: pub fields
        Ok(Statement::StructStatement {
            token,
            name,
            fields,
        })
    }

    fn impl_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(
            TokenType::Impl,
            "expected 'impl' keyword"
        )?;
        let name = self.require(
            TokenType::Identifier,
            "expected a struct name in impl declaration"
        )?;

        self.require(
            TokenType::LeftBrace,
            "expected '{' after impl name"
        )?;

        let mut top_level_statements = vec![];
        let mut impl_statements = vec![];
        loop {
            let Some(def_token) = self.peek() else {
                return Err(self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Identifier,
                    "expected '{' or impl definition"
                )));
            };

            if def_token.token_type == TokenType::RightBrace {
                break;
            }

            let statement = self.impl_pub_statement()?;

            match statement {
                Statement::StaticStatement { .. } | Statement::ConstStatement { .. } =>
                    top_level_statements.push(statement),
                Statement::FnStatement {
                    function, ..
                } => impl_statements.push(function),
                _ => unreachable!(
                    "Unexpected statement while parsing impl statement"
                ),
            }
        }

        self.require(
            TokenType::RightBrace,
            "expected '}' after impl definition"
        )?;

        Ok(Statement::ImplStatement {
            token,
            implemented_type: name,
            top_level_statements,
            functions: impl_statements,
        })
    }

    pub fn expression_statement(&mut self) -> ParserResult<Statement> {
        let expression = self.parse_expression()?;

        self.require(
            TokenType::Semicolon, "expect ';' after expression",
        )?;

        let statement = Statement::ExpressionStatement {
            expression: Box::new(expression)
        };

        Ok(statement)
    }

    fn while_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(
            TokenType::While, "expected 'while' keyword"
        )?;

        let condition = self.parse_expression()?;
        let statements = self.parse_block_of_statements()?;

        let while_statement = Statement::WhileStatement {
            token,
            condition: Box::new(condition),
            body: statements,
        };

        Ok(while_statement)
    }

    fn break_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(
            TokenType::Break, "expected 'break' keyword"
        )?;

        let next_token = match self.advance() {
            Some(token) => token,
            None => return Err(
                self.wrap_error(ParserError::unexpected_eof(TokenType::Semicolon, ""))
            )
        };

        // todo: not tied to actual loop
        if next_token.token_type == TokenType::Identifier {
            self.require(TokenType::Semicolon, "")?;


            Ok(Statement::BreakStatement {
                token,
                loop_key: Some(next_token.clone())
            })
        } else if next_token.token_type == TokenType::Semicolon {
            Ok(Statement::BreakStatement {
                token,
                loop_key: None
            })
        } else {
            Err(
                self.wrap_error(ParserError::unexpected_token(
                    next_token, TokenType::Semicolon,
                    "Expected ';' after 'break' keyword"
                ))
            )
        }
    }

    fn continue_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(TokenType::Continue, "")?;
        let next_token = match self.advance() {
            Some(token) => token,
            None => return Err(
                self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Semicolon,
                    "expected ';' after 'continue' keyword"
                ))
            )
        };
        // todo: not tied to actual loop
        if next_token.token_type == TokenType::Identifier {
            self.require(TokenType::Semicolon, "")?;


            Ok(Statement::BreakStatement {
                token,
                loop_key: Some(next_token.clone())
            })
        } else if next_token.token_type == TokenType::Semicolon {
            Ok(Statement::BreakStatement {
                token,
                loop_key: None
            })
        } else {
            Err(
                self.wrap_error(ParserError::unexpected_token(
                    next_token, TokenType::Semicolon,
                    "expected ';' after 'continue' keyword"
                ))
            )
        }
    }

    fn return_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(TokenType::Return, "")?;

        let expr_token = match self.peek() {
            Some(token) => token,
            None => return Err(
                self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Semicolon,
                    "expected ';' after 'return' keyword"
                ))
            )
        };

        let ret_expr = if expr_token.token_type == TokenType::Semicolon {
            self.advance();

            None
        } else {
            let expression = self.parse_expression()?;
            self.require(TokenType::Semicolon, "")?;

            Some(Box::new(expression))
        };

        Ok(Statement::ReturnStatement {
            token,
            expression: ret_expr,
        })
    }

    fn parse_block_of_statements(&mut self) -> ParserResult<Vec<Statement>> {
        self.require(TokenType::LeftBrace, "")?;
        let mut statements = vec![];

        while let Some(token) = self.peek() {
            if token.token_type == TokenType::RightBrace {
                break;
            }

            let statement = self.statement()?;
            statements.push(statement);
        }
        self.require(TokenType::RightBrace, "")?;

        Ok(statements)
    }

    //// expressions
    fn block_expression(&mut self) -> ParserResult<Expression> {
        let token = self.require(TokenType::LeftBrace, "")?;
        let mut statements = vec![];
        let mut return_expression = None;

        while let Some(token) = self.peek().cloned() {
            if token.token_type == TokenType::RightBrace {
                break;
            }

            let statement = self.statement()?;
            statements.push(statement);

            if token.token_type == TokenType::RightBrace {
                break;
            }

            let expression = self.parse_expression();
            if let Ok(expression) = expression {
                let next_token = match self.peek_ahead(1) {
                    Some(token) => token,
                    _ => return Err(self.wrap_error(ParserError::unexpected_eof(
                        token.token_type.clone(),
                        "expected '}' or ';' after expression"
                    )))
                };

                if next_token.token_type == TokenType::RightBrace {
                    return_expression = Some(Box::new(expression));

                    break
                } else if  next_token.token_type == TokenType::Semicolon {
                    self.advance();

                    statements.push(Statement::ExpressionStatement {
                        expression: Box::new(expression)
                    });

                    continue
                } else {
                    return Err(self.wrap_error(ParserError::unexpected_token(
                        next_token.clone(),
                        TokenType::RightBrace,
                        "Expected '}' after expression at the end of block"
                    )));
                }
            }
        }

        self.require(
            TokenType::RightBrace,
            "Expected '}' at the end of block"
        )?;

        Ok(Expression::Block {
            token,
            statements,
            return_expression,
        })
    }

    fn grouping(&mut self) -> ParserResult<Expression> {
        let token = self.require(
            TokenType::LeftParenthesis,
            "expected '(' in grouping expression"
        )?;
        let expression = self.parse_expression()?;
        self.require(
            TokenType::RightParenthesis,
            "expected ')' in grouping expression"
        )?;

        Ok(Expression::Grouping {
            token,
            expression: Box::new(expression),
        })
    }

    fn identifier(&mut self) -> ParserResult<Expression> {
        let name = self.require(
            TokenType::Identifier, "primary identifier"
        )?;

        Ok(Expression::Identifier {
            name
        })
    }

    fn self_expression(&mut self) -> ParserResult<Expression> {
        let self_token = self.require(
            TokenType::SelfToken, "self primarity identifier"
        )?;

        Ok(Expression::SelfExpression {
            token: self_token,
        })
    }

    fn function_expression(&mut self) -> ParserResult<Expression> {
        let (token, function_name) = self.function_start()?;

        let mut arguments = vec![];
        loop {
            let token = match self.peek().cloned() {
                Some(token) => token,
                None => return Err(self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Identifier,
                    "Expected argument or closing parenthesis"
                )))
            };

            if token.token_type == TokenType::RightParenthesis {
                break;
            }

            let typed_declaration = self.function_argument(token)?;
            arguments.push(typed_declaration);

            let token = match self.peek() {
                Some(token) => token,
                None => return Err(self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Comma,
                    "expected comma or closing parenthesis"
                )))
            };

            if token.token_type != TokenType::Comma {
                break;
            }
            self.advance();
        }

        let (type_identifier, body) = self.function_end()?;

        let function = Function {
            name: function_name,
            arguments,
            return_type: type_identifier,
            body
        };

        Ok(Expression::FnExpression {
            token,
            function
        })
    }

    fn if_else_expression(&mut self) -> ParserResult<Expression> {
        todo!("if else expression is not implemented")
    }

    fn token_as_literal<T>(
        &mut self, token: &Token
    ) -> ParserResult<T>
    where
        T: FromStr,
    {
        T::from_str(token.literal.as_str()).or_else(|err|
            Err(self.wrap_error(ParserError::literal_parse_error(
                token.clone(),
                stringify!(T),
            )))
        )
    }

    fn literal(&mut self) -> ParserResult<Expression> {
        self.advance_skipping_comments();

        let token = match self.peek().cloned() {
            Some(token) => token,
            _ => return Err(self.wrap_error(ParserError::unexpected_eof(
                TokenType::Identifier,
                "expected primary token"
            )))
        };

        let literal = match token.token_type {
            TokenType::U8Literal => Literal::U8 {
                token: token.clone(),
                value: self.token_as_literal(&token)?,
            },
            TokenType::U16Literal => Literal::U16 {
                token: token.clone(),
                value: self.token_as_literal(&token)?,
            },
            TokenType::U32Literal => Literal::U32 {
                token: token.clone(),
                value: self.token_as_literal(&token)?,
            },
            TokenType::U64Literal => Literal::U64 {
                token: token.clone(),
                value: self.token_as_literal(&token)?,
            },
            // },TokenType::U16Literal |
            // TokenType::U32Literal | TokenType::U64Literal |
            // TokenType::I8Literal | TokenType::I16Literal |
            // TokenType::I32Literal | TokenType::I64Literal |
            // TokenType::F32Literal | TokenType::F64Literal |
            // TokenType::BoolLiteral | TokenType::StringLiteral |
            // TokenType::CharLiteral | TokenType::MultilineStringLiteral |
            // TokenType::Vector2Literal | TokenType::Vector3Literal |
            // TokenType::Vector4Literal | TokenType::Matrix2Literal |
            // TokenType::Matrix3Literal | TokenType::Matrix4Literal => {},
            _ => return Err(self.wrap_error(ParserError::UnexpectedPrimaryExpression(
                token
            )))
        };

        Ok(Expression::Literal(literal))
    }
    fn primary(&mut self) -> ParserResult<Expression> {
        self.advance_skipping_comments();

        let current_token = match self.peek() {
            Some(token) => token,
            _ => return Err(self.wrap_error(ParserError::unexpected_eof(
                TokenType::Identifier,
                "expected primary token"
            )))
        };

        match current_token.token_type {
            TokenType::Identifier => self.identifier(),
            TokenType::RightParenthesis => self.grouping(),
            TokenType::If => self.if_else_expression(),
            TokenType::Fn => self.function_expression(),
            TokenType::SelfToken => self.self_expression(),
            _ => self.literal()
        }
    }

    fn dot(&mut self) -> ParserResult<Expression> {
        self.primary()
    }

    fn call(&mut self) -> ParserResult<Expression> {
        self.dot()
    }

    fn array_slice(&mut self) -> ParserResult<Expression> {
        let mut expr = self.call()?;

        loop {
            let Some(open_token) = self.match_current(
                TokenType::LeftBracket, "array slice"
            )? else {
                return Ok(expr);
            };

            self.advance_and_skip_next_comments();

            let slice_expression = self.parse_expression()?;

            let Some(_) = self.match_current(
                TokenType::RightBracket, "array slice closing bracket"
            )? else {
                return Err(self.wrap_error(ParserError::UnterminatedArraySlice(
                    open_token
                )));
            };

            self.advance_and_skip_next_comments();

            expr = Expression::ArraySlice {
                token: open_token,
                array_expression: Box::new(expr),
                slice_expression: Box::new(slice_expression),
            };
        }
    }

    fn unary(&mut self) -> ParserResult<Expression> {
        let Some(token) = self.match_tokens(
            TokenType::UNARY_OPERATORS, "unary operator"
        )? else {
            return self.array_slice();
        };

        // todo: maybe error: parse array slice
        let expression = self.parse_expression()?;

        Ok(Expression::Unary {
            token: token.clone(),
            operator: token,
            expression: Box::new(expression)
        })
    }

    fn postfix(&mut self) -> ParserResult<Expression> {
        self.unary()
    }

    fn parse_left_associative<F>(
        &mut self, parse_operand: F, operators: &[TokenType]
    ) -> ParserResult<Expression>
    where
        F: Fn(&mut Self) -> ParserResult<Expression>
    {
        let mut expr = parse_operand(self)?;

        while let Some(operator) = self.match_tokens(operators, "binary expression")?
        {
            let right = parse_operand(self)?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right)
            };
        }

        Ok(expr)
    }

    fn multiplicative(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::postfix,
            TokenType::MULTIPLICATIVE_OPERATORS,
        )
    }

    fn additive(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::multiplicative,
            TokenType::ADDITIVE_OPERATORS,
        )
    }

    fn binary_shifts(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::additive,
            TokenType::BINARY_SHIFT_OPERATORS,
        )
    }

    fn bitwise_and(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::binary_shifts,
            &[TokenType::BinaryAnd]
        )
    }

    fn bitwise_xor(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::bitwise_and,
            &[TokenType::BinaryXor]
        )
    }

    fn bitwise_or(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::bitwise_xor,
            &[TokenType::BinaryOr]
        )
    }

    fn comparisons(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::bitwise_or,
            TokenType::COMPARISON_OPERATORS,
        )
    }

    fn logical_and(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::comparisons,
            &[TokenType::LogicalAnd]
        )
    }

    fn logical_or(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::logical_and,
            &[TokenType::LogicalOr]
        )
    }

    fn ranges(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::logical_or,
            TokenType::RANGE_OPERATORS,
        )
    }

    pub fn parse_expression(&mut self) -> ParserResult<Expression> {
        self.ranges()
    }
}


/////////////////////////////////////////

fn create_test_parser(filepath: &'static str) -> Parser {
    println!("cwd: {}", env::current_dir().unwrap().display());
    let file = read_to_string(Path::new(filepath)).unwrap();
    let mut lexer = Lexer::new(file);
    let tokens = lexer.lex();

    let parser = Parser::new(tokens);

    parser
}

fn ttoken(
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

fn tidentifier(lexeme: &'static str) -> Token {
    Token::new(
        TokenType::Identifier,
        String::from(lexeme),
        String::from(lexeme),
        0,
        0,
    )
}

fn tsemicolon() -> Token {
    Token::new(
        TokenType::Semicolon,
        String::from(";"),
        String::default(),
        0,
        0,
    )
}

fn primitive_type_as_str(token_type: TokenType) -> &'static str {
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
        TokenType::V2 => "v2",
        TokenType::V3 => "v3",
        TokenType::V4 => "v4",
        _ => panic!("can only stringify primitive types")
    }
}

macro_rules! ttypean {
    ($lit:literal $(, $is_mut:expr)?) => {
        TypeAnnotation {
            kind: TypeKind::Simple(
                Type { name: ttoken(TokenType::Identifier, $lit, $lit) }
            ),
            // name: ttoken(TokenType::Identifier, $lit, $lit),
            is_mut: false $(|| $is_mut )?,
        }
    };
    ($t:path $(, $is_mut:expr)?) => {
        TypeAnnotation {
            kind: TypeKind::Simple(
                Type { name: ttoken($t, primitive_type_as_str($t), "") }
            ),
            // name: ttoken($t, primitive_type_as_str($t), ""),
            is_mut: false $(|| $is_mut)?,
        }
    };
}

macro_rules! ttypedecl {
    ($name:expr, $lit:literal $(, $is_mut:expr)?) => {
        TypedDeclaration {
            name: tidentifier($name),
            declared_type: ttypean!($lit $(, $is_mut)?),
        }
    };
    ($name:expr, $t:path $(, $is_mut:expr)?) => {
        TypedDeclaration {
            name: tidentifier($name),
            declared_type: ttypean!($t $(, $is_mut)?),
        }
    };
}

macro_rules! ttypeanptr {
     ($lit:literal, $points_to_mut:expr $(, $is_mut:expr)?) => {
        TypeAnnotation {
            kind: TypeKind::Pointer(
                PointerAnnotation {
                    inner_type: Box::new(TypeKind::Simple(Type {
                        name: ttoken(TokenType::Identifier, $lit, $lit),
                    })),
                    points_to_mut: $points_to_mut
                },
            ),
            is_mut: false $(|| $is_mut )?,
        }
    };
    ($t:path, $points_to_mut:expr $(, $is_mut:expr)?) => {
        TypeAnnotation {
            kind: TypeKind::Pointer(
                PointerAnnotation {
                    inner_type: Box::new(TypeKind::Simple(Type {
                        name: ttoken($t, primitive_type_as_str($t), ""),
                    })),
                    points_to_mut: $points_to_mut
                }
            ),
            is_mut: false $(|| $is_mut)?,
        }
    };
}

macro_rules! ttypedeclptr {
    ($name:expr, $lit:literal, $points_to_mut:expr $(, $is_mut:expr)?) => {
        TypedDeclaration {
            name: tidentifier($name),
            declared_type: ttypeanptr!(
                $lit, $points_to_mut $(, $is_mut)?
            )
        }
    };
    ($name:expr, $t:path, $points_to_mut:expr $(, $is_mut:expr)?) => {
        TypedDeclaration {
            name: tidentifier($name),
            declared_type: ttypeanptr!(
                $t, $points_to_mut $(, $is_mut)?
            )
        }
    };
}

mod tests {
    use rstest::*;
    use crate::syntax::traits::PartialTreeEq;
    use super::*;

    pub fn assert_tree_eq(
        expected: Vec<Statement>, actual: Vec<Statement>
    ) {
        let ea = expected.iter().zip(actual).enumerate();

        for (i, (expected_statement, actual_statement)) in ea {
            assert!(
                expected_statement.partial_eq(&actual_statement),
                "{}th statement;\n{:?}\n!=\n{:?}",
                i, expected_statement, actual_statement
            );
        }
    }

    #[rstest]
    #[
        case(
            "./resources/simple/struct_no_fields.lr",
            vec![
                Statement::StructStatement {
                    token: ttoken(TokenType::Struct, "struct", ""),
                    name: tidentifier("TestNoFields"),
                    fields: vec![],
                }
            ]
        )
    ]
    #[
        case(
            "./resources/simple/struct_simple_fields.lr",
            vec![
                Statement::StructStatement {
                    token: ttoken(TokenType::Struct, "struct", ""),
                    name: tidentifier("TestStructWithSimpleFields"),
                    fields: vec![
                        ttypedecl!("test_field_i8", TokenType::I8),
                        ttypedecl!("test_field_i16", TokenType::I16),
                        ttypedecl!("test_field_i32", TokenType::I32),
                        ttypedecl!("test_field_i64", TokenType::I64),
                        ttypedecl!("test_field_u8", TokenType::U8),
                        ttypedecl!("test_field_u16", TokenType::U16),
                        ttypedecl!("test_field_u32", TokenType::U32),
                        ttypedecl!("test_field_u64", TokenType::U64),
                        ttypedecl!("test_field_f32", TokenType::F32),
                        ttypedecl!("test_field_f64", TokenType::F64),
                        ttypedecl!("test_field_bool", TokenType::Bool),
                    ]
                }
            ]
        )
    ]
    #[
        case(
            "./resources/simple/struct_compound_fields.lr",
             vec![
                Statement::StructStatement {
                    token: ttoken(TokenType::Struct, "struct", ""),
                    name: tidentifier("TypeA"),
                    fields: vec![
                        ttypedecl!("type_a_field_1", TokenType::I32),
                    ]
                },
                Statement::StructStatement {
                    token: ttoken(TokenType::Struct, "struct", ""),
                    name: tidentifier("TestCompound"),
                    fields: vec![
                        ttypedecl!("type_a", "TypeA"),
                        ttypedecl!("simple_i32", TokenType::I32),
                    ]
                }
            ]
        )
    ]
    #[
        case(
            "./resources/simple/struct_pointer_fields.lr",
            vec![
                Statement::StructStatement {
                    token: ttoken(TokenType::Struct, "struct", ""),
                    name: tidentifier("TestStructA"),
                    fields: vec![
                        ttypedecl!("test_field_i8", TokenType::I8),
                    ]
                },
                Statement::StructStatement {
                    token: ttoken(TokenType::Struct, "struct", ""),
                    name: tidentifier("TestStructWithSimpleFields"),
                    fields: vec![
                        ttypedeclptr!(
                            "test_field_i8", TokenType::I8, false
                        ),
                        ttypedeclptr!("test_field_i16", TokenType::I16, true),
                        ttypedeclptr!("test_field_i32", TokenType::I32, false),
                        TypedDeclaration {
                            name: tidentifier("test_field_i64"),
                            declared_type: TypeAnnotation {
                                kind: TypeKind::Pointer(PointerAnnotation {
                                    inner_type: Box::new(TypeKind::Pointer(
                                        PointerAnnotation {
                                            inner_type: Box::new(TypeKind::Pointer(
                                                PointerAnnotation {
                                                    inner_type: Box::new(
                                                        TypeKind::Simple(Type {
                                                            name: ttoken(TokenType::I64, "i64", ""),
                                                        })
                                                    ),
                                                    points_to_mut: true,
                                                }
                                            )),
                                            points_to_mut: false,
                                        }
                                    )),
                                    points_to_mut: true,
                                }),
                                is_mut: false,
                            }
                        },
                        ttypedeclptr!("test_field_test_struct_a", "TestStructA", true),
                    ]
               },
            ]
        )
    ]
    #[
        case(
            "./resources/simple/fn_no_args_no_return_empty.lr",
            vec![
                Statement::FnStatement {
                    token: ttoken(TokenType::Fn, "fn", ""),
                    function: ImplFunction::Function(Function {
                        name: tidentifier("test"),
                        arguments: vec![],
                        return_type: None,
                        body: vec![],
                    })
                },
            ]
        )
    ]
    #[
        case(
            "./resources/simple/fn_no_args_simple_return_empty.lr",
             vec![
                Statement::FnStatement {
                    token: ttoken(TokenType::Fn, "fn", ""),
                    function: ImplFunction::Function(Function {
                        name: tidentifier("test"),
                        arguments: vec![],
                        return_type: Some(ttypean!(TokenType::I32)),
                        body: vec![],
                    })
                },
                Statement::StructStatement {
                    token: ttoken(TokenType::Struct, "struct", ""),
                    name: tidentifier("TestA"),
                    fields: vec![],
                },
                Statement::FnStatement {
                    token: ttoken(TokenType::Fn, "fn", ""),
                    function: ImplFunction::Function(Function {
                        name: tidentifier("test2"),
                        arguments: vec![],
                        return_type: Some(ttypean!("TestA")),
                        body: vec![],
                    })
                },
            ]
        )
    ]
    #[
        case(
            "./resources/simple/fn_simple_args_simple_return_empty.lr",
            vec![
                Statement::FnStatement {
                    token: ttoken(TokenType::Fn, "fn", ""),
                    function: ImplFunction::Function(Function {
                        name: tidentifier("test"),
                        arguments: vec![
                            ttypedecl!("a_i32", TokenType::I32),
                            ttypedecl!("b_i64", TokenType::I64),
                        ],
                        return_type: Some(ttypean!(TokenType::I32)),
                        body: vec![],
                    })
                },
                Statement::StructStatement {
                    token: ttoken(TokenType::Struct, "struct", ""),
                    name: tidentifier("TestA"),
                    fields: vec![],
                },
                Statement::FnStatement {
                    token: ttoken(TokenType::Fn, "fn", ""),
                    function: ImplFunction::Function(Function {
                        name: tidentifier("test2"),
                        arguments: vec![
                            ttypedecl!("a1", TokenType::I8),
                            ttypedecl!("a2", TokenType::I16),
                            ttypedecl!("a3", TokenType::I32),
                            ttypedecl!("a4", TokenType::I64),
                            ttypedecl!("a5", TokenType::U8),
                            ttypedecl!("a6", TokenType::U16, true),
                            ttypedecl!("a7", TokenType::U32, true),
                            ttypedecl!("a8", TokenType::U64, true),
                            ttypedecl!("a9", "TestA", true),
                        ],
                        return_type: Some(ttypean!("TestA")),
                        body: vec![],
                    })
                },
            ]
        )
    ]
    #[
        case(
            "./resources/simple/fn_simple_args_simple_return_simple_body.lr",
            vec![
                Statement::FnStatement {
                    token: ttoken(TokenType::Fn, "fn", ""),
                    function: ImplFunction::Function(Function {
                        name: tidentifier("test"),
                        arguments: vec![
                            ttypedecl!("a_i32", TokenType::I32),
                            ttypedecl!("b_i64", TokenType::I64),
                        ],
                        return_type: Some(ttypean!(TokenType::I32)),
                        body: vec![
                            Statement::ReturnStatement {
                                token: ttoken(TokenType::Return, "return", ""),
                                expression: Some(Box::new(Expression::Literal(Literal::I32 {
                                    token: ttoken(TokenType::I32Literal, "i32", ""),
                                    value: 5,
                                })))
                            }
                        ],
                    })
                },
                Statement::StructStatement {
                    token: ttoken(TokenType::Struct, "struct", ""),
                    name: tidentifier("TestA"),
                    fields: vec![],
                },
                Statement::FnStatement {
                    token: ttoken(TokenType::Fn, "fn", ""),
                    function: ImplFunction::Function(Function {
                        name: tidentifier("test2"),
                        arguments: vec![
                            ttypedecl!("a1", TokenType::I8),
                            ttypedecl!("a2", TokenType::I16),
                            ttypedecl!("a3", TokenType::I32),
                            ttypedecl!("a4", TokenType::I64),
                            ttypedecl!("a5", TokenType::U8),
                            ttypedecl!("a6", TokenType::U16, true),
                            ttypedecl!("a7", TokenType::U32, true),
                            ttypedecl!("a8", TokenType::U64, true),
                            ttypedecl!("a9", "TestA", true),
                        ],
                        return_type: Some(ttypean!("TestA")),
                        body: vec![],
                    })
                },
            ]
        )
    ]
    // #[case("./resources/simple/fn_pointer_args_pointer_return_simple_body.lr")]
    #[
        case(
            "./resources/simple/impl_empty.lr",
            vec![
                Statement::StructStatement {
                    token: ttoken(TokenType::Struct, "struct", ""),
                    name: tidentifier("Test"),
                    fields: vec![],
                },
                Statement::ImplStatement {
                    token: ttoken(TokenType::Impl, "impl", ""),
                    implemented_type: tidentifier("Test"),
                    top_level_statements: vec![],
                    functions: vec![],
                }
            ]
        )
    ]
    #[
        case(
            "./resources/simple/impl_simple.lr",
            vec![
                Statement::StructStatement {
                    token: ttoken(TokenType::Struct, "struct", ""),
                    name: tidentifier("Test"),
                    fields: vec![],
                },
                Statement::ImplStatement {
                    token: ttoken(TokenType::Impl, "impl", ""),
                    implemented_type: tidentifier("Test"),
                    top_level_statements: vec![],
                    functions: vec![
                        ImplFunction::Function(Function {
                            name: tidentifier("function"),
                            arguments: vec![],
                            return_type: None,
                            body: vec![],
                        }),
                        ImplFunction::Function(Function {
                            name: tidentifier("function2"),
                            arguments: vec![
                                ttypedecl!("a", TokenType::I32),
                            ],
                            return_type: Some(ttypean!(TokenType::I64)),
                            body: vec![
                                Statement::ReturnStatement {
                                    token: ttoken(TokenType::Return, "return", ""),
                                    expression: Some(Box::new(Expression::Literal(Literal::I32 {
                                        token: ttoken(TokenType::I32Literal, "i32", ""),
                                        value: 5,
                                    })))
                                }
                            ],
                        }),
                    ],
                }
            ]
        )
    ]
    // #[case("./resources/simple/impl_methods_and_functions.lr")]
    // #[case("./resources/simple/impl_consts_and_statics_methods_and_functions.lr")]
    pub fn test_simple_statements(
        #[case] source_code: &'static str,
        #[case] expected_statements: Vec<Statement>,
    ) {
        let mut parser = create_test_parser(source_code);

        let ast = parser.parse_panic();

        assert_tree_eq(expected_statements, ast);
    }
}