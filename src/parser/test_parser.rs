use rstest::*;
use crate::syntax::ast::Statement;
use crate::syntax::traits::PartialTreeEq;
use crate::utils::test_utils::*;
use crate::syntax::tokens::*;
use crate::syntax::ast::*;
use crate::*;

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
                    ttypedecl!("test_field_char", TokenType::Char),
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
                            expression: Some(Box::new(tbinary!("a", TokenType::Star, "b"))),
                        }
                    ],
                })
            },
            Statement::StructStatement {
                token: ttoken(TokenType::Struct, "struct", ""),
                name: tidentifier("TestA"),
                fields: vec![
                    ttypedecl!("a", TokenType::I32),
                ],
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
                        ttypedecl!("a6", TokenType::U16),
                        ttypedecl!("a7", TokenType::U32),
                        ttypedecl!("a8", TokenType::U64),
                        ttypedecl!("a9", "TestA"),
                    ],
                    return_type: Some(ttypean!("TestA")),
                    body: vec![
                        Statement::ExpressionStatement {
                            expression: Box::new(Expression::Grouping {
                                token: ttoken(TokenType::LeftParenthesis, "(", ""),
                                expression: Box::new(tbinary!("a1", TokenType::Plus, "a1")),
                            })
                        },
                        Statement::ReturnStatement {
                            token: ttoken(TokenType::Return, "return", ""),
                            expression: Some(Box::new(Expression::StructInitializer {
                                token: ttoken(TokenType::LeftBrace, "{", ""),
                                struct_name: tidentifier("TestA"),
                                field_initializers: vec![],
                            })),
                        }
                    ],
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
                                expression: Some(Box::new(Expression::Cast {
                                    token: ttoken(TokenType::As, "as", ""),
                                    left: Box::new(Expression::Identifier {
                                        name: tidentifier("a"),
                                    }),
                                    target_type: ttypean!(TokenType::I64),
                                    is_reinterpret_cast: false,
                                }))
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