use crate::compiler::c_transpiler::CTranspilerSemantics;
use crate::parser::semantics::scope_resolving::ScopeResolvingSemantics;
use crate::syntax::ast::Statement;
use crate::syntax::ast::*;
use crate::syntax::tokens::*;
use crate::syntax::traits::PartialTreeEq;
use crate::typing::type_validation::TypeValidationSemantics;
use crate::utils::test_utils::*;
use crate::*;
use rstest::*;

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
// #[
//     case(
//         "./resources/simple/struct_no_fields.lr",
//         vec![
//             Statement::new_struct(
//                 ttoken(TokenType::Struct, "struct", ""),
//                 tidentifier("TestNoFields"),
//                 vec![],
//             )
//         ]
//     )
// ]
// #[
//     case(
//         "./resources/simple/struct_simple_fields.lr",
//         vec![
//             Statement::new_struct(
//                 ttoken(TokenType::Struct, "struct", ""),
//                 tidentifier("TestStructWithSimpleFields"),
//                 vec![
//                     ttypedecl!("test_field_i8", TokenType::I8),
//                     ttypedecl!("test_field_i16", TokenType::I16),
//                     ttypedecl!("test_field_i32", TokenType::I32),
//                     ttypedecl!("test_field_i64", TokenType::I64),
//                     ttypedecl!("test_field_u8", TokenType::U8),
//                     ttypedecl!("test_field_u16", TokenType::U16),
//                     ttypedecl!("test_field_u32", TokenType::U32),
//                     ttypedecl!("test_field_u64", TokenType::U64),
//                     ttypedecl!("test_field_f32", TokenType::F32),
//                     ttypedecl!("test_field_f64", TokenType::F64),
//                     ttypedecl!("test_field_bool", TokenType::Bool),
//                     ttypedecl!("test_field_char", TokenType::Char),
//                 ]
//             )
//         ]
//     )
// ]
// #[
//     case(
//         "./resources/simple/struct_compound_fields.lr",
//          vec![
//             Statement::new_struct(
//                 ttoken(TokenType::Struct, "struct", ""),
//                 tidentifier("TypeA"),
//                 vec![
//                     ttypedecl!("type_a_field_1", TokenType::I32),
//                 ]
//             ),
//             Statement::new_struct(
//                 ttoken(TokenType::Struct, "struct", ""),
//                 tidentifier("TestCompound"),
//                 vec![
//                     ttypedecl!("type_a", "TypeA"),
//                     ttypedecl!("simple_i32", TokenType::I32),
//                 ]
//             )
//         ]
//     )
// ]
// #[
//     case(
//         "./resources/simple/struct_pointer_fields.lr",
//         vec![
//             Statement::new_struct(
//                 ttoken(TokenType::Struct, "struct", ""),
//                 tidentifier("TestStructA"),
//                 vec![
//                     ttypedecl!("test_field_i8", TokenType::I8),
//                 ]
//             ),
//             Statement::new_struct(
//                 ttoken(TokenType::Struct, "struct", ""),
//                 tidentifier("TestStructWithSimpleFields"),
//                 vec![
//                     ttypedeclptr!(
//                         "test_field_i8", TokenType::I8, false
//                     ),
//                     ttypedeclptr!("test_field_i16", TokenType::I16, true),
//                     ttypedeclptr!("test_field_i32", TokenType::I32, false),
//                     TypedDeclaration::new(
//                         tidentifier("test_field_i64"),
//                         TypeAnnotation::new(
//                             TypeKind::Pointer(PointerAnnotation {
//                                 inner_type: Box::new(TypeKind::Pointer(
//                                     PointerAnnotation {
//                                         inner_type: Box::new(TypeKind::Pointer(
//                                             PointerAnnotation {
//                                                 inner_type: Box::new(
//                                                     TypeKind::Simple(Type {
//                                                         name: ttoken(TokenType::I64, "i64", ""),
//                                                     })
//                                                 ),
//                                                 points_to_mut: true,
//                                             }
//                                         )),
//                                         points_to_mut: false,
//                                     }
//                                 )),
//                                 points_to_mut: true,
//                             }),
//                             false,
//                         )
//                     ),
//                     ttypedeclptr!("test_field_test_struct_a", "TestStructA", true),
//                 ]
//            ),
//         ]
//     )
// ]
// #[
//     case(
//         "./resources/simple/fn_no_args_no_return_empty.lr",
//         vec![
//             Statement::new_fn(
//                 ttoken(TokenType::Fn, "fn", ""),
//                 ImplFunction::Function(Function {
//                     name: tidentifier("test"),
//                     arguments: vec![],
//                     return_type: None,
//                     body: vec![],
//                 })
//             ),
//         ]
//     )
// ]
// #[
//     case(
//         "./resources/simple/fn_no_args_simple_return_empty.lr",
//          vec![
//             Statement::new_fn(
//                 ttoken(TokenType::Fn, "fn", ""),
//                 ImplFunction::Function(Function {
//                     name: tidentifier("test"),
//                     arguments: vec![],
//                     return_type: Some(ttypean!(TokenType::I32)),
//                     body: vec![],
//                 })
//             ),
//             Statement::new_struct(
//                 ttoken(TokenType::Struct, "struct", ""),
//                 tidentifier("TestA"),
//                 vec![],
//             ),
//             Statement::new_fn(
//                 ttoken(TokenType::Fn, "fn", ""),
//                 ImplFunction::Function(Function {
//                     name: tidentifier("test2"),
//                     arguments: vec![],
//                     return_type: Some(ttypean!("TestA")),
//                     body: vec![],
//                 })
//             ),
//         ]
//     )
// ]
// #[
//     case(
//         "./resources/simple/fn_simple_args_simple_return_empty.lr",
//         vec![
//             Statement::new_fn(
//                 ttoken(TokenType::Fn, "fn", ""),
//                 ImplFunction::Function(Function {
//                     name: tidentifier("test"),
//                     arguments: vec![
//                         ttypedecl!("a_i32", TokenType::I32),
//                         ttypedecl!("b_i64", TokenType::I64),
//                     ],
//                     return_type: Some(ttypean!(TokenType::I32)),
//                     body: vec![],
//                 })
//             ),
//             Statement::new_struct(
//                 ttoken(TokenType::Struct, "struct", ""),
//                 tidentifier("TestA"),
//                 vec![],
//             ),
//             Statement::new_fn(
//                 ttoken(TokenType::Fn, "fn", ""),
//                 ImplFunction::Function(Function {
//                     name: tidentifier("test2"),
//                     arguments: vec![
//                         ttypedecl!("a1", TokenType::I8),
//                         ttypedecl!("a2", TokenType::I16),
//                         ttypedecl!("a3", TokenType::I32),
//                         ttypedecl!("a4", TokenType::I64),
//                         ttypedecl!("a5", TokenType::U8),
//                         ttypedecl!("a6", TokenType::U16, true),
//                         ttypedecl!("a7", TokenType::U32, true),
//                         ttypedecl!("a8", TokenType::U64, true),
//                         ttypedecl!("a9", "TestA", true),
//                     ],
//                     return_type: Some(ttypean!("TestA")),
//                     body: vec![],
//                 })
//             ),
//         ]
//     )
// ]
#[
    case(
        "./resources/simple/fn_simple_args_simple_return_simple_body.lr",
        vec![
            Statement::new_fn(
                ttoken(TokenType::Fn, "fn", ""),
                ImplFunction::Function(Function {
                    name: tidentifier("test"),
                    arguments: vec![
                        ttypedecl!("a_i32", TokenType::I32),
                        ttypedecl!("b_i64", TokenType::I64),
                    ],
                    return_type: Some(ttypean!(TokenType::I32)),
                    body: vec![
                        Statement::new_return(
                            ttoken(TokenType::Return, "return", ""),
                            Some(Box::new(
                                Expression::new_binary(
                                    Expression::new_grouping(
                                        ttoken(TokenType::LeftParenthesis, "(", ""),
                                        Expression::new_cast(
                                            ttoken(TokenType::As, "as", ""),
                                            Expression::new_identifier(
                                                tidentifier("a_i32")
                                            ),
                                            ttypean!(TokenType::I64),
                                            false
                                        )
                                    ),
                                    ttoken(TokenType::Star, "*", ""),
                                    Expression::new_identifier(tidentifier("b_i64"))
                                )
                            )),
                        )
                    ],
                })
            ),
            Statement::new_struct(
                ttoken(TokenType::Struct, "struct", ""),
                tidentifier("TestA"),
                vec![
                    ttypedecl!("a", TokenType::I32),
                ],
            ),
            Statement::new_fn(
                ttoken(TokenType::Fn, "fn", ""),
                ImplFunction::Function(Function {
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
                        Statement::new_expression(
                            Expression::new_grouping(
                                ttoken(TokenType::LeftParenthesis, "(", ""),
                                tbinary!("a1", TokenType::Plus, "a1"),
                            )
                        ),
                        Statement::new_return(
                            ttoken(TokenType::Return, "return", ""),
                            Some(Box::new(Expression::new_struct_initializer(
                                ttoken(TokenType::LeftBrace, "{", ""),
                                tidentifier("TestA"),
                                vec![],
                            ))),
                        )
                    ],
                })
            ),
        ]
    )
]
// #[case("./resources/simple/fn_pointer_args_pointer_return_simple_body.lr")]
// #[
//     case(
//         "./resources/simple/impl_empty.lr",
//         vec![
//             Statement::new_struct(
//                 ttoken(TokenType::Struct, "struct", ""),
//                 tidentifier("Test"),
//                 vec![],
//             ),
//             Statement::new_impl(
//                 ttoken(TokenType::Impl, "impl", ""),
//                 tidentifier("Test"),
//                 vec![],
//                 vec![],
//             )
//         ]
//     )
// ]
// #[
//     case(
//         "./resources/simple/impl_simple.lr",
//         vec![
//             Statement::new_struct(
//                 ttoken(TokenType::Struct, "struct", ""),
//                 tidentifier("Test"),
//                 vec![],
//             ),
//             Statement::new_impl(
//                 ttoken(TokenType::Impl, "impl", ""),
//                 tidentifier("Test"),
//                 vec![],
//                 vec![
//                     FnStatement::new(
//                         ttoken(TokenType::Fn, "fn", ""),
//                         ImplFunction::Function(Function {
//                             name: tidentifier("function"),
//                             arguments: vec![],
//                             return_type: None,
//                             body: vec![],
//                         })
//                     ),
//                     FnStatement::new(
//                         ttoken(TokenType::Fn, "fn", ""),
//                         ImplFunction::Function(Function {
//                             name: tidentifier("function2"),
//                             arguments: vec![
//                                 ttypedecl!("a", TokenType::I32),
//                             ],
//                             return_type: Some(ttypean!(TokenType::I64)),
//                             body: vec![
//                                 Statement::new_return(
//                                     ttoken(TokenType::Return, "return", ""),
//                                     Some(Box::new(Expression::new_cast(
//                                         ttoken(TokenType::As, "as", ""),
//                                         Expression::new_identifier(
//                                             tidentifier("a"),
//                                         ),
//                                         ttypean!(TokenType::I64),
//                                         false,
//                                     )))
//                                 )
//                             ],
//                         }),
//                     )
//                 ],
//             )
//         ]
//     )
// ]
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

#[rstest]
#[case("./resources/simple/fn_no_args_no_return_empty.lr")]
#[case("./resources/simple/fn_no_args_simple_return_empty.lr")]
#[case("./resources/simple/fn_pointer_args_pointer_return_simple_body.lr")]
#[case("./resources/simple/fn_simple_args_simple_return_empty.lr")]
#[case("./resources/simple/fn_simple_args_simple_return_simple_body.lr")]
#[case("./resources/simple/impl_consts_and_statics_methods_and_functions.lr")]
#[case("./resources/simple/impl_empty.lr")]
#[case("./resources/simple/impl_methods_and_functions.lr")]
#[case("./resources/simple/impl_simple.lr")]
#[case("./resources/simple/struct_compound_fields.lr")]
#[case("./resources/simple/struct_no_fields.lr")]
#[case("./resources/simple/struct_pointer_fields.lr")]
#[case("./resources/simple/struct_simple_fields.lr")]
#[case("./resources/example.rs")]
pub fn test_semantics(#[case] source_code: &'static str) {
    println!("Test case: {}", source_code);
    let mut parser = create_test_parser(source_code);
    parser.panic_on_error = true;
    let (_, _, statements) = parser.parse();

    let flatten_tree_analyzer = SemanticsAnalyzer::new(&statements)
        .with_semantics::<FlattenTree>();
    let semantic_analyzer = SemanticsAnalyzer::new(&statements)
        .with_semantics::<FlowControlSemantics>()
        .with_semantics::<NameScopingSemantics>();
    let second_semantic_analyzer = SemanticsAnalyzer::new(&statements)
        .with_semantics::<ScopeResolvingSemantics>()
        .with_semantics::<TypeValidationSemantics>();
    // let second_pass_analyzer = SemanticsAnalyzer::new(&statements)
    //     .with(name_resolving_semanti);

    let ast_context = ParserContext::default()
        .analyze_by(flatten_tree_analyzer)
        .then_analyze_by(semantic_analyzer)
        .then_analyze_by(second_semantic_analyzer)
        .then_analyze_by(
            SemanticsAnalyzer::new(&statements)
                .with_semantics::<CTranspilerSemantics>()
        );
    if current_id() != AstNodeIndex(ast_context.parser.ast_nodes.len()) {
        let mut kv: Vec<_> = ast_context
            .parser
            .ast_nodes
            .iter()
            .collect();
        kv.sort_by(|a, b| a.0.0.cmp(&b.0.0));

        println!("Skipped nodes:");
        for i in 1..kv.len() {
            if kv[i].0.0 - kv[i - 1].0.0 > 1 {
                println!("skip:");
                if i - 3 < kv.len() {
                    println!("key: {}\n\t{:?}", kv[i - 3].0, kv[i - 3].1);
                }
                if i - 2 < kv.len() {
                    println!("key: {}\n\t{:?}", kv[i - 2].0, kv[i - 2].1);
                }
                println!("key: {}\n\t{:?}", kv[i - 1].0, kv[i - 1].1);
                println!("key: {}\n\t{:?}", kv[i].0, kv[i].1);

                if i + 1 < kv.len() {
                    println!("key: {}\n\t{:?}", kv[i + 1].0, kv[i + 1].1);
                }
                println!("skip end");
            }
        }

        println!("all nodes:");
        for (key, value) in kv {
            println!(
                "key: {}\n\t{:?}",
                key, value
            );
        }
    }
    assert_eq!(
        current_id().0,
        ast_context.parser.ast_nodes.len(),
        "test"
    );

    println!("done {}", source_code);
}