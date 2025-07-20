use crate::syntax::tokens::{TokenType, DOUBLE_KEYWORD_TOKENS, DOUBLE_TOKENS, KEYWORDS, SIMPLE_TOKENS};
use crate::dev_assert;
use maplit::{hashmap, hashset};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;
use crate::syntax::traits::PartialTreeEq;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Token {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,

    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: String,
}


impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}:{}:{} at {}:{}",
            self.token_type, self.lexeme, self.literal,
            self.start_line, self.start_column
        )
    }
}

impl Token {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn new_with<T>(builder: T) -> Self
    where
        T: FnOnce(&mut Self)
    {
        let mut token = Self::default();
        token.replace(builder);

        token
    }

    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: String,
        start_line: usize,
        start_column: usize,
    ) -> Self {
        Self {
            start_line,
            start_column,
            end_line: start_line,
            end_column: start_column,
            token_type,
            lexeme,
            literal,
        }
    }

    pub fn replace<T>(&mut self, builder: T)
    where
        T: FnOnce(&mut Self)
    {
        builder(self);
    }

    pub fn with<T>(&mut self, builder: T) -> Self

    where
        T: FnOnce(&mut Self)
    {
        self.replace(builder);

        self.clone()
    }

    pub fn at(&self) -> String {
        format!(
            "{}:{}",
            self.start_line,
            self.start_column
        )
    }

    pub fn at_from(&self) -> String {
        format!(
            "{} to {}:{}",
            self.at(),
            self.end_line,
            self.end_column
        )
    }
}

impl PartialTreeEq for Token {
    type Other = Token;
    fn partial_eq(&self, other: &Token) -> bool {
        self.token_type == other.token_type &&
            self.lexeme == other.lexeme &&
            self.literal == other.literal
    }
}

pub struct Lexer {
    stream: String,
    start_position: usize,
    current_position: usize,

    token_start_line: usize,
    token_start_column: usize,
    current_line: usize,
    current_column: usize,
}

impl Lexer {
    pub fn new(stream: String) -> Self {
        Self {
            stream,
            start_position: 0,
            current_position: 0,
            token_start_line: 1,
            token_start_column: 1,
            current_line: 1,
            current_column: 1,
        }
    }

    #[inline(always)]
    fn at(&self) -> String {
        format!("{}:{}", self.current_line, self.current_column)
    }

    #[inline(always)]
    pub fn is_at_end(&self) -> bool {
        self.current_position >= self.stream.len()
    }

    #[inline(always)]
    pub fn peek(&self) -> char {
        let c = self.stream.chars().nth(self.current_position);

        c.unwrap_or_else(|| '\0')
    }

    #[inline(always)]
    pub fn peek_ahead(&self, offset: usize) -> char {
        let c = self.stream.chars().nth(self.current_position + offset);

        c.unwrap_or_else(|| '\0')
    }

    #[inline(always)]
    pub fn peek_skip_whitespaces(&mut self) -> Option<char> {
        while self.peek().is_whitespace() {
            self.advance();
        }

        if self.peek() == '\0' {
            None
        } else {
            Some(self.peek())
        }
    }

    #[inline(always)]
    pub fn advance(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        let c = self.peek();

        if c == '\n' {
            self.current_line += 1;
            self.current_column = 1;
        }

        self.current_position += 1;
        self.current_column += 1;

        c
    }

    #[inline(always)]
    pub fn require(&mut self, required_character: char) -> char {
        let at = self.at();
        let c = self.advance();

        if c != required_character {
            panic!(
                "Expected character {} at {} but got {}",
                required_character, at,
                c
            );
        }

        c
    }

    #[inline(always)]
    fn pin_position(&mut self) {
        self.token_start_line = self.current_line;
        self.token_start_column = self.current_column;
    }

    #[inline(always)]
    fn extract(&self, start: usize, end: usize) -> String {
        dev_assert!(start < self.stream.len(), "Invalid start position");

        self.stream[start..end].to_string()
    }

    #[inline(always)]
    fn up_to_current_position(&self) -> String {
        self.extract(self.start_position, self.current_position)
    }

    #[inline(always)]
    fn up_to_current_position_from(&self, start: usize) -> String {
        self.extract(start, self.current_position)
    }


    fn skip_whitespaces(&mut self) {
        loop {
            let c = self.peek();
            
            if c == '\0' || !c.is_whitespace() {
                break;
            }

            self.advance();
        }
    }

    fn parse_single_line_comment(&mut self) -> String {
        let start = self.current_position;
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }

        self.advance();

        self.up_to_current_position_from(start)
    }

    fn parse_multi_line_comment(&mut self) -> String {
        let start = self.current_position;

        loop {
            if self.is_at_end() {
                panic!(
                    "Unexpected end of stream while parsing multi line comment: {}",
                    self.at()
                );
            }

            let c = self.peek();

            if c == '*' && self.peek_ahead(1) == '/' {
                let comment_literal = self.up_to_current_position_from(start);

                self.advance();
                self.advance();

                return comment_literal;
            }

            self.advance();
        }
    }

    pub fn parse_double_token(&mut self) -> Option<Token> {
        let c = self.peek_skip_whitespaces()?;
        let token_variants = DOUBLE_TOKENS.get(&c)?;

        self.advance();

        let c = self.peek();

        let next_token_type = token_variants.0.get(&c);

        let token_type = if let Some(double_token_type) = next_token_type {
            self.advance();

            *double_token_type
        } else {
            token_variants.1
        };

        let literal = match token_type {
            TokenType::SingleLineComment => self.parse_single_line_comment(),
            TokenType::MultilineCommentStart => self.parse_multi_line_comment(),
            TokenType::MultilineCommentEnd => panic!(
                "Unexpected multiline comment at {}",
                self.at()
            ),
            _ => String::default()
        };

        let token = self.make_token(token_type, literal);

        Some(token)
    }

    pub fn parse_simple_token(&mut self) -> Option<Token> {
        let c = self.peek_skip_whitespaces()?;
        
        self.pin_position();
        
        let token_type = SIMPLE_TOKENS.get(&c)?;
        
        self.advance();
        
        let token = self.make_token(*token_type,  String::default());

        Some(token)
    }


    #[inline(always)]
    pub fn make_token(&self, token_type: TokenType, literal: String) -> Token {
        Token::new_with(|t| {
            t.token_type = token_type;
            t.start_line = self.token_start_line;
            t.end_line = self.current_line;
            t.start_column = self.token_start_column;
            t.end_column = self.current_column;

            t.lexeme = self.up_to_current_position();
            t.literal = literal;
        })
    }

    pub fn parse_identifier(&mut self) -> Token {
        let mut chars = vec![];

        while self.peek().is_alphanumeric() || self.peek() == '_' {
            let c = self.advance();
            chars.push(c);
        }

        let identifier = String::from_iter(&chars);

        if let Some(token_type) = KEYWORDS.get(&identifier.as_str()) {
            self.make_token(*token_type, String::default())
        } else {
            self.make_token(TokenType::Identifier, identifier)
        }
    }


    fn extract_digits(&mut self) -> String {
        let start = self.current_position;

        let mut digits = vec![];
        while self.peek().is_ascii_digit() {
            let digit = self.advance();
            digits.push(digit);
        }

        self.up_to_current_position_from(start)
    }

    fn parse_number_postfix(&mut self) -> Option<TokenType> {
        static NUMBER_POSTFIXES: LazyLock<HashMap<&str, TokenType>> = LazyLock::new(
            || hashmap! {
                "i8" => TokenType::I8Literal,
                "i16" => TokenType::I16Literal,
                "i32" => TokenType::I32Literal,
                "i64" => TokenType::I64Literal,
                "u8" => TokenType::U8Literal,
                "u16" => TokenType::U16Literal,
                "u32" => TokenType::U32Literal,
                "u64" => TokenType::U64Literal,
                "f32" => TokenType::F32Literal,
                "f64" => TokenType::F64Literal,
            }
        );
        static POSTFIX_START_CHARS: LazyLock<HashSet<char>> = LazyLock::new(
            || hashset! { 'i', 'u', 'f'}
        );

        if !POSTFIX_START_CHARS.contains(&self.peek()) {
            return None;
        }

        let postfix_start = self.advance();

        let postfix_number = self.extract_digits();
        let postfix = format!("{}{}", postfix_start, postfix_number); // TODO: pizdets

        let postfix_literal_type = match NUMBER_POSTFIXES.get(&postfix.as_str()) {
            Some(literal_type) => *literal_type,
            _ => panic!("Unexpected number literal postfix: {} at {}", postfix, self.at())
        };

        Some(postfix_literal_type)
    }

    pub fn parse_number(&mut self) -> Token {
        let is_signed = if self.peek() == '-' {
            self.advance();

            true
        } else {
            false
        };
        let mut token_type = if is_signed {
            TokenType::I32
        } else {
            TokenType::U32
        };

        let _decimal_part = self.extract_digits();

        // TODO: duplicated & maybe will be changed if literals are reworked
        if self.peek() == '.' {
            token_type = TokenType::F32;
            let _float_point = self.advance();
            let _floating_point_part = self.extract_digits();

            let parsed_literal_type = self.parse_number_postfix();

            if let Some(literal_type) = parsed_literal_type {
                token_type = match literal_type {
                    TokenType::F32Literal | TokenType::F64Literal => literal_type,
                    _ => panic!(
                        "Excepted f32 or f64 number postfix for floating point value at {}",
                        self.at()
                    )
                };

                let number_literal = self.extract(
                    self.start_position, self.current_position - 3
                );

                self.make_token(
                    token_type, number_literal
                )
            } else {
                let number_literal = self.up_to_current_position();

                self.make_token(token_type, number_literal)
            }
        } else {
            let parsed_literal_type = self.parse_number_postfix();

            if let Some(literal_type) = parsed_literal_type {
                token_type = match literal_type {
                    TokenType::F32Literal | TokenType::F64Literal => literal_type,
                    TokenType::U8Literal | TokenType::U16Literal |
                    TokenType::U32Literal | TokenType::U64Literal => {
                        if is_signed {
                            panic!(
                                "Unexpected unsigned integer postfix for signed integer at {}",
                                self.at()
                            );
                        }
                        literal_type
                    },
                    _ => literal_type,
                };

                let postfix_length = match token_type {
                    TokenType::I8Literal | TokenType::U8Literal => 2,
                    _ => 3
                };

                let number_literal = self.extract(
                    self.start_position, self.current_position - postfix_length
                );

                self.make_token(token_type, number_literal)
            } else {
                self.make_token(token_type, self.up_to_current_position())
            }
        }
    }

    pub fn parse_character_literal(&mut self) -> Token {
        static ESCAPE_CHARS: LazyLock<HashSet<char>> = LazyLock::new(|| hashset! {
            'a', 'b', 'e', 'f',
            'n', 'r', 't', 'v',
            '\\', '\'',
        });

        let start = self.current_position;

        self.require('\'');

        // TODO: code mark; if literals are reworked
        let _ = if self.peek() == '\\' {
            let escaped_c = self.advance();

            if !ESCAPE_CHARS.contains(&escaped_c) {
                panic!(
                    "Unexpected escaped character {} at {}",
                    escaped_c, self.at()
                );
            }

            escaped_c
        } else {
            let c = self.advance();

            if c == '\'' {
                panic!("Empty character literal at {}", self.at());
            }

            c
        };

        self.require('\'');

        let literal = self.extract(start + 1, self.current_position - 1);

        self.make_token(TokenType::CharLiteral, literal)
    }

    pub fn parse_string_literal(&mut self) -> Token {
        let start = self.current_position;
        self.require('"');

        // TODO: code mark; if literals are reworked
        loop {
            if self.is_at_end() {
                panic!("Unterminated string literal at {}", self.at());
            }

            let c = self.advance();

            if c == '\n' {
                panic!("Unterminated string literal at {}", self.at());
            }

            if c == '"' {
                break;
            }
        }

        let string_literal = self.extract(start + 1, self.current_position - 1);

        self.make_token(TokenType::StringLiteral, string_literal)
    }

    pub fn parse_multiline_string_literal(&mut self) -> Token {
        let start = self.current_position;
        self.require('`');

        // TODO: code mark; if literals are reworked
        loop {
            if self.is_at_end() {
                panic!("Unterminated string literal at {}", self.at());
            }

            let c = self.advance();

            if c == '`' {
                break;
            }
        }

        let string_literal = self.stream[start + 1..self.current_position - 1]
            .to_string();
        self.make_token(TokenType::StringLiteral, string_literal)
    }

    #[inline(always)]
    fn can_start_identifier(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }
    
    fn try_convert_to_double_keyword(&mut self, token: Token) -> (Token, Option<Token>) {
        let Some(double_keyword) = DOUBLE_KEYWORD_TOKENS.get(
            &token.token_type
        ) else {
            return (token, None);
        };
        self.skip_whitespaces();
        
        if !Lexer::can_start_identifier(self.peek()) {
            return (token, None);
        }
        
        self.start_position = self.current_position;
        self.pin_position();
        
        let next_identifier = self.parse_identifier();
            
        if next_identifier.token_type != double_keyword.0 {
            return (token, Some(next_identifier));
        }
        
        let new_keyword = self.make_token(
            double_keyword.1,
            [token.literal, next_identifier.literal].join(" ")
        );
        
        (new_keyword, None)
    }
    pub fn next_token(&mut self) -> Option<Token> {
        if self.is_at_end() {
            return None;
        }

        let c = self.peek();
        if c.is_whitespace() {
            self.skip_whitespaces();
        }

        self.start_position = self.current_position;
        self.pin_position();

        let token = if let Some(token) = self.parse_simple_token() {
            token
        } else if let Some(token) = self.parse_double_token() {
            token
        } else if Lexer::can_start_identifier(c) { 
            self.parse_identifier()
        } else if c.is_ascii_digit() {
            self.parse_number()
        } else {
            match c {
                '\'' => self.parse_character_literal(),
                '"' => self.parse_string_literal(),
                '`' => self.parse_multiline_string_literal(),
                _ => {
                    panic!("Unknown character: {} at {}", c, self.at())
                }
            }
        };

        Some(token)
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespaces();
            let token = self.next_token();
            
            match token {
                Some(token) => {
                    let (first, second) = self.try_convert_to_double_keyword(
                        token
                    );
                    tokens.push(first);
                    
                    if let Some(second) = second {
                        tokens.push(second);
                    }
                },
                None => {
                    break;
                }
            }
            self.skip_whitespaces();
        }
        
        tokens.push(self.make_token(TokenType::EOF, "".to_string()));
      
        tokens
    }
}