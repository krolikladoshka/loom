use crate::parser::errors::ParserError;
use crate::syntax::ast::{Ast, AstNode, AstNodeIndex, Expression, Function, ImplFunction, Literal, Method, PointerAnnotation, Statement, Type, TypeAnnotation, TypeKind, TypedDeclaration};
use crate::syntax::lexer::Token;
use crate::syntax::tokens::TokenType;
use std::collections::HashMap;
use std::str::FromStr;
use crate::dev_assert;

pub type ParserResult<T> = Result<T, ParserError>;

pub enum AstRef<'a> {
    Expression(&'a Expression),
    Statement(&'a Statement)
}

pub struct Parser<'a> {
    pub tokens: Vec<Token>,
    start_position: usize,
    current_position: usize,
    pub(crate) panic_on_error: bool,
    node_id_counter: AstNodeIndex,
    pub ast_nodes: HashMap<AstNodeIndex, AstRef<'a>>
}


impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            start_position: 0,
            current_position: 0,
            panic_on_error: false,
            node_id_counter: AstNodeIndex(0),
            ast_nodes: HashMap::with_capacity(100),
        }
    }
    
    #[inline(always)]
    pub fn current_node_id(&self) -> usize {
        self.node_id_counter.clone().into()
    }
    
    #[inline(always)]
    fn next_node_id(&mut self) -> usize {
        self.node_id_counter.increment().into()
        // let current_node_id = self.node_id_counter;
        // self.node_id_counter += 1;
        // 
        // current_node_id
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
        let token = self.advance()?;
        self.advance_skipping_comments();

        Some(token)
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
            TokenType::Bool, TokenType::Char,
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

    pub(crate) fn parse_panic(&mut self) -> Vec<Statement> {
        self.panic_on_error = true;

        let (result, error, parse_results) = self.parse();
        
        assert!(error.is_none(), "parser returned an error during panic mode");
        dev_assert!(parse_results.iter().all(|r| r.is_ok()), "parser returned an error during panic mode");
        
        result
    }

    pub fn parse_next(&mut self) -> ParserResult<Statement> {
        if self.is_at_end() {
            return Err(ParserError::Eof);
        }

        self.advance_skipping_comments();

        let result = self.top_level_statement();
        
        if result.is_err() {
            self.advance_until_next_statement();
        }
        self.advance_skipping_comments();
        
        result
    }
    
    pub fn parse(&mut self)
        -> (Vec<Statement>, Option<Vec<ParserError>>, Vec<ParserResult<Statement>>)
    {
        let mut parser_results = vec![];
        let mut ast = vec![];
        let mut errors = vec![];

        loop {
            match self.parse_next() {
                Ok(statement) => {
                    ast.push(statement.clone());
                    parser_results.push(Ok(statement));
                },
                Err(err) => {
                    if let ParserError::Eof = err {
                        break;
                    }
                    parser_results.push(Err(err.clone()));
                    errors.push(err);
                    self.advance_until_next_statement();
                    self.empty_statement();
                },
            }
        }

        let error = if errors.is_empty() {
            None
        } else {
            Some(errors)
        };

        (ast, error, parser_results)
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

        Ok(Statement::new_empty(token))
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

    fn impl_pub_statement(&mut self, implemented_type: Token) -> ParserResult<Statement> {
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
        let statement = self.impl_variable_declarations_and_functions(
            implemented_type
        )?;
        // let statement = self.variable_declarations_and_functions()?;

        Ok(statement)
    }
    
    fn impl_variable_declarations_and_functions(
        &mut self,
        implemented_type: Token,
    ) -> ParserResult<Statement> {
        self.advance_skipping_comments();
        let token = self.peek_protected()?.clone();

        match token.token_type {
            TokenType::Static => self.static_variable_statement(),
            TokenType::Const => self.const_variable_statement(),
            TokenType::Fn => self.impl_function_statement(implemented_type),
            _ => Err(self.wrap_error(ParserError::UnexpectedStatement(token))),
        }
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

    fn defer_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(
            TokenType::Defer,
            "expected 'defer' keyword"
        )?;

        let expr = self.parse_expression()?;
        match expr {
            Expression::Call {..} | Expression::MethodCall {..} |
            Expression::Identifier {..} => Ok(
                Statement::new_defer(token, expr, false)
            ),
            _ => Err(self.wrap_error(ParserError::defer_non_callable_argument(
                token
            )))
        }
    }

    fn defer_block_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(
            TokenType::DeferBlock,
            "expected 'defer block' keyword"
        )?;

        let expr = self.parse_expression()?;
        match expr {
            Expression::Call {..} | Expression::MethodCall {..} |
            Expression::Identifier {..} => Ok(
                Statement::new_defer(token, expr, true)
            ),
            _ => Err(self.wrap_error(ParserError::defer_non_callable_argument(
                token
            )))
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
            TokenType::Defer => self.defer_statement(), // self.defer_statement(),
            TokenType::DeferBlock => self.defer_block_statement(),
            TokenType::Use => todo!("use statement"), // self.use_statement(),
            TokenType::If => self.if_else_statement(),
            TokenType::Loop => self.loop_statement(),
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

            Ok(TypeAnnotation::new(
                TypeKind::Pointer(pointer_type),
                is_mut,
            ))
        } else {
            let simple_type = self.simple_type()?;

            Ok(TypeAnnotation::new(
                TypeKind::Simple(simple_type),
                is_mut,
            ))
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

                self.advance_and_skip_next_comments();
                Some(self.type_annotation(no_mut)?)
        } else {
            None
        };

        let token = self.peek_protected()?;
        let initializer = if token.token_type == TokenType::Equals {
            self.advance_and_skip_next_comments();

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

        let mut is_mut = false;
        if let Some(_) = self.match_current(
            TokenType::Mut, "expected mut modifier"
        )? {
            self.advance_and_skip_next_comments();
            is_mut = true;
        }

        let result = self.common_variable_declaration(
            false,
            |identifier, type_annotation, initializer|
                Ok(Statement::new_let(
                    let_token,
                    identifier,
                    type_annotation,
                    initializer,
                    is_mut
                ))
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

        let mut is_mut = false;
        if let Some(_) = self.match_current(
            TokenType::Mut, "expected mut modifier"
        )? {
            self.advance_and_skip_next_comments();
            is_mut = true;
        }

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

                Ok(Statement::new_static(
                    token,
                    identifier,
                    annotation,
                    initializer,
                    is_mut
                ))
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

                Ok(Statement::new_const(
                    token,
                    identifier,
                    annotation,
                    initializer,
                ))
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

        Ok(TypedDeclaration::new(
            argument_name,
            annotation,
        ))
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

            if arguments.len() < 1 && token.token_type == TokenType::Star {
                // if token.token_type == TokenType::Star {
                self.advance();
                    // is_bound_method = true
                // }
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
            self.advance();
        }

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


        Ok(Statement::new_fn(token, function))
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

        Ok(Statement::new_fn(token, function))
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

                    Ok(TypedDeclaration::new(
                        identifier,
                        annotation,
                    ))
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
        Ok(Statement::new_struct(token, name, fields))
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

            let statement = self.impl_pub_statement(name.clone())?;

            match statement {
                Statement::StaticStatement { .. } | Statement::ConstStatement { .. } =>
                    top_level_statements.push(statement),
                Statement::FnStatement(fn_statement) =>
                    impl_statements.push(fn_statement),
                _ => unreachable!(
                    "Unexpected statement while parsing impl statement"
                ),
            }
        }

        self.require(
            TokenType::RightBrace,
            "expected '}' after impl definition"
        )?;

        Ok(Statement::new_impl(
            token,
            name,
            top_level_statements,
            impl_statements,
        ))
    }

    pub fn expression_statement(&mut self) -> ParserResult<Statement> {
        let expression = self.parse_expression()?;

        self.require(
            TokenType::Semicolon, "expect ';' after expression",
        )?;

        let statement = Statement::new_expression(expression);

        Ok(statement)
    }

    fn if_else_statement(&mut self) -> ParserResult<Statement> {
        let if_token = self.require(
            TokenType::If,
            "expected 'if' keyword"
        )?;

        self.advance_skipping_comments();

        let condition = self.parse_before_block_expression()?;
        let statements = self.parse_block_of_statements()?;

        let else_block = if let Some(else_token) = self.match_current(
            TokenType::Else,
            "expect 'else keyword"
        )? {
            self.advance_and_skip_next_comments();

            if let Some(_) = self.match_current(
                TokenType::If,
                "expect 'if' keyword"
            )? {
                let if_else_statement = self.if_else_statement()?;

                Some(vec![if_else_statement])
            } else {
                Some(self.parse_block_of_statements()?)
            }
        } else {
            None
        };

        Ok(Statement::new_if_else(
            if_token,
            condition,
            statements,
            else_block,
        ))
    }
    fn loop_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(
            TokenType::Loop, "expected 'loop' keyword"
        )?;
        let statements = self.parse_block_of_statements()?;

        Ok(Statement::new_while(
            token.clone(),
            Expression::new_literal(Literal::new_bool(
                token, // TODO: wlel
                true
            )),
            statements
        ))
    }

    fn while_statement(&mut self) -> ParserResult<Statement> {
        let token = self.require(
            TokenType::While, "expected 'while' keyword"
        )?;

        let condition = self.parse_before_block_expression()?;
        let statements = self.parse_block_of_statements()?;

        let while_statement = Statement::new_while(
            token,
            condition,
            statements,
        );

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


            Ok(Statement::new_break(
                token,
                Some(next_token.clone())
            ))
        } else if next_token.token_type == TokenType::Semicolon {
            Ok(Statement::new_break(token, None))
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


            Ok(Statement::new_continue(
                token,
                Some(next_token.clone())
            ))
        } else if next_token.token_type == TokenType::Semicolon {
            Ok(Statement::new_continue(token, None))
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

        Ok(Statement::new_return(token, ret_expr))
    }

    fn parse_block_of_statements(&mut self) -> ParserResult<Vec<Statement>> {
        self.require(TokenType::LeftBrace, "'{' is required at the start of the block")?;
        let mut statements = vec![];

        loop {
            self.advance_skipping_comments();
            let Some(token) = self.peek() else {
                break;
            };
            
            if token.token_type == TokenType::RightBrace {
                break;
            }

            let statement = self.statement()?;
            statements.push(statement);
        }
        self.require(TokenType::RightBrace, "'}' is required at the end of the block")?;

        Ok(statements)
    }

    //// expressions
    fn block_expression(&mut self) -> ParserResult<Expression> {
        let token = self.require(TokenType::LeftBrace, "")?;
        let mut statements = vec![];
        let mut return_expression = None;

        loop {
            self.advance_skipping_comments();
            let Some(token) = self.peek().cloned() else {
                break;
            };
            
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

                    statements.push(Statement::new_expression(expression));

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
        self.advance_skipping_comments();

        Ok(Expression::new_block(
            token,
            statements,
            return_expression,
        ))
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

        Ok(Expression::new_grouping(token, expression))
    }

    fn identifier(&mut self) -> ParserResult<Expression> {
        let name = self.require(
            TokenType::Identifier, "primary identifier"
        )?;

        Ok(Expression::new_identifier(name))
    }

    fn self_expression(&mut self) -> ParserResult<Expression> {
        let self_token = self.require(
            TokenType::SelfToken, "self primarity identifier"
        )?;

        Ok(Expression::new_self(self_token))
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

        Ok(Expression::new_fn(token, function))
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
            TokenType::U8Literal => Literal::new_u8(
                token.clone(), 
                self.token_as_literal(&token)?
            ),
            TokenType::U16Literal => Literal::new_u16(
                token.clone(),
                self.token_as_literal(&token)?,
            ),
            TokenType::U32Literal => Literal::new_u32(
                token.clone(),
                self.token_as_literal(&token)?,
            ),
            TokenType::U64Literal => Literal::new_u64(
                token.clone(),
                self.token_as_literal(&token)?,
            ),
            TokenType::I8Literal => Literal::new_u8(
                token.clone(),
                self.token_as_literal(&token)?
            ),
            TokenType::I16Literal => Literal::new_u16(
                token.clone(),
                self.token_as_literal(&token)?,
            ),
            TokenType::I32Literal => Literal::new_u32(
                token.clone(),
                self.token_as_literal(&token)?,
            ),
            TokenType::I64Literal => Literal::new_u64(
                token.clone(),
                self.token_as_literal(&token)?,
            ),
            TokenType::F32Literal => Literal::new_f32(
                token.clone(),
                self.token_as_literal(&token)?,
            ),
            TokenType::F64Literal => Literal::new_f64(
                token.clone(),
                self.token_as_literal(&token)?,
            ),
            TokenType::StringLiteral => Literal::new_string(
                token.clone(),
                token.literal.clone(),
            ),
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
        self.advance_and_skip_next_comments();

        Ok(Expression::new_literal(literal))
    }

    fn initializer_for(&mut self, struct_name: Token) -> ParserResult<Expression> {
        let token = self.require(
            TokenType::LeftBrace,
            "opening '{' in initializer"
        )?;

        let mut field_initializers = HashMap::new();
        loop {
            if let Some(_) = self.match_current(
                TokenType::RightBrace,
                "closing '}' in initializer"
            )? {
                break;
            }
            let field_name = self.require(
                TokenType::Identifier,
                "field identifier in initializer"
            )?;

            if field_initializers.contains_key(&field_name.literal) {
                return Err(self.wrap_error(ParserError::duplicated_struct_initializer_field(
                    struct_name,
                    field_name
                )));
            }

            self.advance_skipping_comments();
            self.require(
                TokenType::Colon,
                "expected ':' after field in initializer"
            )?;

            let initializer = self.parse_expression()?;
            field_initializers.insert(
                field_name.literal.clone(), (field_name, initializer)
            );

            if self.match_current(
                TokenType::RightBrace, "',' or '}' in initializer"
            )?.is_none() {
                break;
            }
        }

        self.require(
            TokenType::RightBrace,
            "closing '}' after initializer"
        )?;

        Ok(Expression::new_struct_initializer( 
            token,
            struct_name,
            field_initializers.into_values().collect()
        ))
    }

    fn identifier_or_initializer(&mut self) -> ParserResult<Expression> {
        let name = self.require(
            TokenType::Identifier, "primary identifier"
        )?;

        if let Some(_) = self.match_current(
            TokenType::LeftBrace,
            "struct initializer"
        )?
        {
            self.initializer_for(name)
        } else {
            Ok(Expression::new_identifier(name))
        }
    }

    fn as_operator(&mut self, expr: Expression, is_raw: bool) -> ParserResult<Expression> {
        let operator_token = if is_raw {
            self.require(
                TokenType::AsRaw,
                "as raw operator in postfix expression"
            )?
        } else {
            self.require(
                TokenType::As,
                "as operator in postfix expression",
            )?
        };

        self.advance_skipping_comments();

        let target_type = self.type_annotation(false)?;

        Ok(Expression::new_cast(
            operator_token,
            expr,
            target_type,
            is_raw,
        ))
    }

    fn dot_with_op<Op>(&mut self, operand: Op) -> ParserResult<Expression>
    where
        Op: Fn(&mut Self) -> ParserResult<Expression>
    {
        let mut expr = operand(self)?;

        loop {
            let Some(access_operator) = self.match_tokens(
                TokenType::ACCESS_OPERATORS,
                "`.` or `->`"
            )? else {
                return Ok(expr);
            };

            self.advance_and_skip_next_comments();
            let identifier = self.require(
                TokenType::Identifier,
                "expected an identifier after `.` or `->`"
            )?;

            match access_operator.token_type {
                TokenType::Dot => {
                    expr = Expression::new_dot_access(expr, identifier);
                },
                TokenType::Arrow => {
                    expr = Expression::new_arrow_access(expr, identifier);
                },
                _ => return Err(self.wrap_error(ParserError::error(
                    "unreachable parser state in . or -> parsing".to_string(),
                    access_operator.start_line,
                    access_operator.start_column,
                    self.current_position
                )))
            }
        }
    }

    // TODO: basically the same as array slice `higher_precedence` `OP_TOKEN` `expr`* `CLOSE_TOKEN`
    fn call_with_op<Op>(&mut self, operand: Op) -> ParserResult<Expression>
    where
        Op: Fn(&mut Self) -> ParserResult<Expression>
    {
        let mut expr = operand(self)?;

        loop {
            let Some(open_token) = self.match_current(
                TokenType::LeftParenthesis, "function call"
            )? else {
                return Ok(expr);
            };

            self.advance_and_skip_next_comments();

            let mut call_arguments = vec![];
            loop {
                if let Some(_) = self.match_current(
                    TokenType::RightParenthesis, "function call"
                )?
                {
                    break;
                };

                let argument = self.parse_expression()?;
                call_arguments.push(argument);

                if self.match_current(TokenType::Comma, "next parameter")?.is_none() {
                    break;
                }
                self.advance_and_skip_next_comments();
            }

            self.require(
                TokenType::RightParenthesis, "function call end"
            )?;
            expr = Expression::new_call(
                open_token,
                expr,
                call_arguments,
            );
        }
    }

    fn array_slice_with_op<Op>(&mut self, operand: Op) -> ParserResult<Expression>
    where
        Op: Fn(&mut Self) -> ParserResult<Expression>,
    {
        let mut expr = operand(self)?;

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

            expr = Expression::new_array_slice(
                open_token,
                expr,
                slice_expression,
            );
        }
    }

    fn unary_with_op<Op>(&mut self, operand: Op) -> ParserResult<Expression>
    where
        Op: Fn(&mut Self) -> ParserResult<Expression>
    {
        let Some(token) = self.match_tokens(
            TokenType::UNARY_OPERATORS, "unary operator"
        )? else {
            return operand(self);
        };
        
        self.advance_and_skip_next_comments();
        // todo: maybe error: parse array slice
        // let expression = self.parse_expression()?;
        let expression = operand(self)?;

        Ok(Expression::new_unary(
            token.clone(),
            token,
            expression
        ))
    }

    fn postfix_with_op<Op>(&mut self, operand: Op) -> ParserResult<Expression>
    where
        Op: Fn(&mut Self) -> ParserResult<Expression>
    {
        let left = operand(self)?;

        self.advance_skipping_comments();

        if let Some(postfix_token) = self.match_tokens(
            TokenType::POSTFIX_OPERATORS,
            "posfix operator"
        )? {
            match postfix_token.token_type {
                TokenType::As => self.as_operator(left, false),
                TokenType::AsRaw => self.as_operator(left, true),
                _ => Ok(left),
            }
        } else {
            Ok(left)
        }
    }

    fn parse_left_associative<F>(
        &mut self, parse_operand: F, operators: &[TokenType]
    ) -> ParserResult<Expression>
    where
        F: Fn(&mut Self) -> ParserResult<Expression>
    {
        let mut expr = parse_operand(self)?;
        loop {
            self.advance_skipping_comments();
            
            let Some(operator) = self.match_tokens(operators, "binary expression")? else {
                break;
            };
            
            self.advance_and_skip_next_comments();

            let right = parse_operand(self)?;
            expr = Expression::new_binary(
                expr,
                operator,
                right
            );
        }

        Ok(expr)
    }

    pub fn parse_assignment_with_op<Op>(&mut self, operand: Op) -> ParserResult<Expression>
    where
        Op: Fn(&mut Self) -> ParserResult<Expression>
    {
        let left = operand(self)?;

        if let Some(assignment_operator) =  self.match_tokens(
            TokenType::ASSIGNMENT_OPERATORS,
            "assignment operators"
        )?
        {
            self.advance_and_skip_next_comments();

            let right = self.parse_assignment_with_op(operand)?;

            let expression = match assignment_operator.token_type {
                TokenType::Equals => Expression::new_assignment(
                    assignment_operator,
                    left,
                    right,
                ),
                TokenType::PlusEquals | TokenType::MinusEquals |
                TokenType::StarEquals | TokenType::SlashEquals |
                TokenType::PercentEquals | TokenType::BinaryInvertEquals |
                TokenType::BinaryOrEquals | TokenType::BinaryXorEquals |
                TokenType::BinaryAndEquals => Expression::new_inplace_assignment(
                    assignment_operator.clone(),
                    left,
                    assignment_operator,
                    right,
                ),
                _ => return Err(self.wrap_error(ParserError::unexpected_token(
                    assignment_operator,
                    TokenType::Equals,
                    "expected an assignment operator"
                )))
            };

            Ok(expression)
        } else {
            Ok(left)
        }
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
            TokenType::Identifier => self.identifier_or_initializer(),
            TokenType::LeftParenthesis => self.grouping(),
            TokenType::If => self.if_else_expression(),
            TokenType::Fn => self.function_expression(),
            TokenType::SelfToken => self.self_expression(),
            _ => self.literal()
        }
    }

    #[inline(always)]
    fn dot(&mut self) -> ParserResult<Expression> {
        self.dot_with_op(Self::primary)
    }

    #[inline(always)]
    fn call(&mut self) -> ParserResult<Expression> {
        self.call_with_op(Self::dot)
    }

    fn array_slice(&mut self) -> ParserResult<Expression> {
        self.array_slice_with_op(Self::call)
    }

    #[inline(always)]
    fn unary(&mut self) -> ParserResult<Expression> {
        self.unary_with_op(Self::array_slice)
    }

    #[inline(always)]
    fn postfix(&mut self) -> ParserResult<Expression> {
        self.postfix_with_op(Self::unary)
    }

    #[inline(always)]
    fn multiplicative(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::postfix,
            TokenType::MULTIPLICATIVE_OPERATORS,
        )
    }

    #[inline(always)]
    fn additive(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::multiplicative,
            TokenType::ADDITIVE_OPERATORS,
        )
    }

    #[inline(always)]
    fn binary_shifts(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::additive,
            TokenType::BINARY_SHIFT_OPERATORS,
        )
    }

    #[inline(always)]
    fn bitwise_and(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::binary_shifts,
            &[TokenType::BinaryAnd]
        )
    }

    #[inline(always)]
    fn bitwise_xor(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::bitwise_and,
            &[TokenType::BinaryXor]
        )
    }

    #[inline(always)]
    fn bitwise_or(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::bitwise_xor,
            &[TokenType::BinaryOr]
        )
    }

    #[inline(always)]
    fn comparisons(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::bitwise_or,
            TokenType::COMPARISON_OPERATORS,
        )
    }

    #[inline(always)]
    fn logical_and(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::comparisons,
            &[TokenType::LogicalAnd]
        )
    }

    #[inline(always)]
    fn logical_or(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::logical_and,
            &[TokenType::LogicalOr]
        )
    }

    #[inline(always)]
    fn ranges(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::logical_or,
            TokenType::RANGE_OPERATORS,
        )
    }

    #[inline(always)]
    fn parse_assignment(&mut self) -> ParserResult<Expression> {
        self.parse_assignment_with_op(Self::ranges)
    }

    #[inline(always)]
    pub fn parse_expression(&mut self) -> ParserResult<Expression> {
        // self.ranges()
        let expression = self.parse_assignment()?;
        self.ast_nodes.insert(expression.get_node_id(), AstRef::Expression(&expression));

        Ok(expression)
    }

    fn primary_before_block(&mut self) -> ParserResult<Expression> {
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
            TokenType::LeftParenthesis => self.grouping(),
            TokenType::If => self.if_else_expression(),
            TokenType::Fn => self.function_expression(),
            TokenType::SelfToken => self.self_expression(),
            _ => self.literal()
        }
    }

    #[inline(always)]
    fn dot_before_block(&mut self) -> ParserResult<Expression> {
        self.dot_with_op(Self::primary_before_block)
    }

    #[inline(always)]
    fn call_before_block(&mut self) -> ParserResult<Expression> {
        self.call_with_op(Self::dot_before_block)
    }

    #[inline(always)]
    fn array_slice_before_block(&mut self) -> ParserResult<Expression> {
        self.call_with_op(Self::call_before_block)
    }

    #[inline(always)]
    fn unary_before_block(&mut self) -> ParserResult<Expression> {
        self.unary_with_op(Self::array_slice_before_block)
    }

    #[inline(always)]
    fn postfix_before_block(&mut self) -> ParserResult<Expression> {
        self.postfix_with_op(Self::unary_before_block)
    }

    #[inline(always)]
    fn multiplicative_before_block(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::postfix_before_block,
            TokenType::MULTIPLICATIVE_OPERATORS,
        )
    }

    #[inline(always)]
    fn additive_before_block(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::multiplicative_before_block,
            TokenType::ADDITIVE_OPERATORS,
        )
    }

    #[inline(always)]
    fn binary_shifts_before_block(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::additive_before_block,
            TokenType::BINARY_SHIFT_OPERATORS,
        )
    }

    #[inline(always)]
    fn bitwise_and_before_block(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::binary_shifts_before_block,
            &[TokenType::BinaryAnd]
        )
    }

    #[inline(always)]
    fn bitwise_xor_before_block(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::bitwise_and_before_block,
            &[TokenType::BinaryXor]
        )
    }

    #[inline(always)]
    fn bitwise_or_before_block(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::bitwise_xor_before_block,
            &[TokenType::BinaryOr]
        )
    }

    #[inline(always)]
    fn comparisons_before_block(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::bitwise_or_before_block,
            TokenType::COMPARISON_OPERATORS,
        )
    }

    #[inline(always)]
    fn logical_and_before_block(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::comparisons_before_block,
            &[TokenType::LogicalAnd]
        )
    }

    #[inline(always)]
    fn logical_or_before_block(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::logical_and_before_block,
            &[TokenType::LogicalOr]
        )
    }

    #[inline(always)]
    pub fn ranges_before_block(&mut self) -> ParserResult<Expression> {
        self.parse_left_associative(
            Self::logical_or_before_block,
            TokenType::RANGE_OPERATORS,
        )
    }

    #[inline(always)]
    fn parse_assignment_before_block(&mut self) -> ParserResult<Expression> {
        self.parse_assignment_with_op(Self::ranges_before_block)
    }
    
    #[inline(always)]
    pub fn parse_before_block_expression(&mut self) -> ParserResult<Expression> {
        self.parse_assignment_before_block()
    }
}
