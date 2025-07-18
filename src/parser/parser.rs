use std::env;
use std::fmt::Display;
use std::fs::read_to_string;
use std::path::Path;
use crate::syntax::ast::{Expression, Function, ImplFunction, Method, Statement, TypeAnnotation, TypedDeclaration};
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
pub enum ParserError {
    EmptyStream,
    UnrecognizedToken(Token),
    UnexpectedToken(UnexpectedTokenError),
    UnexpectedEof(UnexpectedEofError),
    UnexpectedTopLevelStatement(Token),
    UnexpectedStatement(Token),
    TypeAnnotationRequired(TypeAnnotationRequiredError),
    InitializerRequired(InitializerRequiredError),
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

    fn match_tokens(&mut self, token_types: &[TokenType]) -> bool {
        let current_token = match self.peek() {
            Some(token) => token.token_type,
            None => return false,
        };

        token_types.contains(&current_token)
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

    fn type_annotation(&mut self, no_mut: bool) -> ParserResult<TypeAnnotation> {
        let mut is_mut = false;
        if !no_mut {
            let Some(token) = self.peek().cloned() else {
                return Err(self.wrap_error(ParserError::unexpected_eof(
                    TokenType::Identifier,
                    "expected type annotation"
                )));
            };

            if token.token_type == TokenType::Mut {
                self.advance();

                is_mut = true;
            };
        }

        let type_identifier = self.require(
            TokenType::Identifier,
            "expected type identifier"
        )?;

        Ok(TypeAnnotation {
            name: type_identifier,
            is_mut,
        })
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

    fn function_end(&mut self) -> ParserResult<(Option<Token>, Vec<Statement>)>
    {
        self.require(
            TokenType::RightParenthesis,
            "expected ')' after arguments list"
        )?;

        let type_identifier = match self.peek() {
            Some(token) => if token.token_type == TokenType::Arrow {
                self.advance();

                Some(self.require(
                    TokenType::Identifier,
                    "expected return type identifier after '->'"
                )?)
            } else {
                None
            },
            None => None
        };

        self.require(
            TokenType::LeftBrace,
            "expected '{' after function arguments list"
        )?;

        let body = self.parse_block_of_statements()?;

        self.require(
            TokenType::RightBrace,
            "expected '}' after function body"
        )?;

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


    fn primary(&mut self) -> ParserResult<Expression> {
        let current_token = match self.peek() {
            Some(token) => token,
            _ => panic!("Expected primary token")
        };

        match current_token.token_type {
            TokenType::EOF => panic!("Unexpected end of file"),
            _ => todo!("Primary parsing")
        }
    }

    fn dot(&mut self) -> ParserResult<Expression> {
        self.primary()
    }

    fn call(&mut self) -> ParserResult<Expression> {
        self.dot()
    }

    fn array_slice(&mut self) -> ParserResult<Expression> {
        self.call()
    }

    fn unary(&mut self) -> ParserResult<Expression> {
        self.array_slice()
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

        while self.match_tokens(operators) {
            let operator = self.peek_protected()?.clone();
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
            &[TokenType::Star, TokenType::Slash, TokenType::Percent]
        )
    }

    fn additive(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::multiplicative,
            &[TokenType::Plus, TokenType::Minus]
        )
    }

    fn binary_shifts(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::additive,
            &[TokenType::BinaryShiftLeft, TokenType::BinaryShiftRight]
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
            &[
                TokenType::Less, TokenType::LessEqual,
                TokenType::Greater, TokenType::GreaterEqual,
                TokenType::EqualsEquals, TokenType::NotEquals,
            ]
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
            &[TokenType::RangeInclusive, TokenType::RangeExclusive]
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
                        TypedDeclaration {
                            name: tidentifier("test_field_i8"),
                            declared_type: TypeAnnotation {
                                name: ttoken(TokenType::I8, "i8", ""),
                                is_mut: false,
                            },
                        },
                        TypedDeclaration {
                            name: tidentifier("test_field_i16"),
                            declared_type: TypeAnnotation {
                                name: ttoken(TokenType::I8, "i16", ""),
                                is_mut: false,
                            },
                        },
                    ]
                }
            ]
        )
    ]
    // #[case("./resources/simple/struct_compound_fields.lr")]
    // #[case("./resources/simple/struct_pointer_fields.lr")]
    // #[case("./resources/simple/fn_no_args_no_return_empty.lr")]
    // #[case("./resources/simple/fn_no_args_simple_return_empty.lr")]
    // #[case("./resources/simple/fn_simple_args_simple_return_empty.lr")]
    // #[case("./resources/simple/fn_simple_args_simple_return_simple_body.lr")]
    // #[case("./resources/simple/fn_pointer_args_pointer_return_simple_body.lr")]
    // #[case("./resources/simple/impl_empty.lr")]
    // #[case("./resources/simple/impl_simple.lr")]
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