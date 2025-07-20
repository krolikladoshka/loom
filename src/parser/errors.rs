use std::fmt::Display;
use crate::syntax::lexer::Token;
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
pub struct DuplicatedStructInitializerFieldError {
    pub struct_name: Token,
    pub field_name: Token,
}

impl Display for DuplicatedStructInitializerFieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "duplicated struct initializer field {} for {}",
            self.field_name, self.struct_name
        )
    }
}

#[derive(Debug, Clone)]
pub struct UnexpectedArgumentError {
    pub receiver: &'static str,
    pub argument: Token,
}

impl Display for UnexpectedArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{} received unexpected argument {}",
            self.receiver, self.argument
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
    DuplicatedStructInitializerField(DuplicatedStructInitializerFieldError),
    LiteralParseError(LiteralParseError),
    UnexpectedPrimaryExpression(Token),
    UnterminatedArraySlice(Token),
    UnexpectedArgument(UnexpectedArgumentError),
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
                write!(f, "unexpected top level statement: {}", utls),
            Self::UnexpectedStatement(us) =>
                write!(f, "Unexpected statement {}", us),
            Self::TypeAnnotationRequired(t) =>
                write!(f, "{}", t),
            Self::InitializerRequired(t) =>
                write!(f, "{}", t),
            Self::DuplicatedStructInitializerField(d) =>
                write!(f, "{}", d),
            Self::LiteralParseError(err) =>
                write!(f, "{}", err),
            Self::UnexpectedPrimaryExpression(t) =>
                write!(f, "unexpected primary expression token {}", t),
            Self::UnterminatedArraySlice(t) =>
                write!(f, "unterminated array slice for {}", t),
            Self::UnexpectedArgument(t) =>
                write!(f, "{}", t),
            Self::ParseError(t) => write!(f, "Uncategorized error: {}", t),
            Self::CompoundError(errors) => {
                // let mut messages = vec![];
                for (i, error) in errors.iter().enumerate() {
                    writeln!(
                        f, "{}:\n\t{}",
                        i + 1, error
                    )?;
                    // messages.push(write!(f, "{}:\n\t{}", i + 1, *error));
                }
                Ok(())
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

    pub fn duplicated_struct_initializer_field(struct_name: Token, field_name: Token) -> Self {
        Self::DuplicatedStructInitializerField(DuplicatedStructInitializerFieldError {
            struct_name, field_name
        })
    }

    pub fn unexpected_argument(receiver: &'static str, argument: Token) -> Self {
        Self::UnexpectedArgument(UnexpectedArgumentError { receiver, argument })
    }

    pub fn defer_non_callable_argument(argument: Token) -> Self {
        Self::unexpected_argument("defer", argument)
    }
}