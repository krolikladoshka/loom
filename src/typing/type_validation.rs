use std::collections::HashMap;
use std::thread::scope;
use crate::dev_assert;
use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::parser::semantics::SecondSemanticsPassContext;
use crate::syntax::ast::{ArrowAccess, Assignment, AstNode, AstNodeIndex, Binary, Call, Cast, DotAccess, Expression, FnStatement, Function, Grouping, Identifier, InplaceAssignment, LetStatement, LiteralNode, Method, Statement, StructInitializer, StructStatement, TypeAnnotation, TypeKind, TypedDeclaration, Unary};
use crate::typing::literal_typing::{match_binary_op, match_inplace_assignment_op, match_unary_op, verify_cast_operator, BuiltinType, InnerTypeEq, PointerType, StructType, Type};

#[derive(Debug, Clone)]
pub struct TypeValidationContext {
    pub evaluated_types: HashMap<AstNodeIndex, Type>
}

impl TypeValidationContext {
    pub fn new() -> Self {
        Self {
            evaluated_types: HashMap::new()
        }
    }
}

impl Default for TypeValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeValidationContext {
    #[inline(always)]
    fn check_nodes_type_equality(&self, left: &AstNodeIndex, right: &AstNodeIndex) -> bool {
        self.evaluated_types[left] == self.evaluated_types[right]
    }
}

impl AstContext for TypeValidationContext {}

#[derive(Default)]
pub struct TypeValidationSemantics;

impl TypeValidationSemantics {
    fn check_assignment_to_mut(&self, lhs: &Expression, context: &SecondSemanticsPassContext) {
        /*
        let x = 3;
        x = 4; // fail

        let ptr_x: *i32  = &x;
        *ptr_x = 5; // fail because ptr_x is not pointing to mut
        let ptr_x_mut: *mut i32 = &mut x; // maybe fail here? can't &mut a constant variable
        //but
        let ptr_x_mut: *mut i32 = ptr_x as raw *mut i32; // is okay

        *ptr_x_mut  = 5; // okay, what else can i do?
           ptr_x_mut - *mut i32
           *ptr_x_mut - i32
         struct A { x: i32 }
         let a = A { x: 46 };
         a.x = 5; // should fail, a is immutable, but a.x is evaluated as i32
         // check whether it's a dot access? if so then check variable of structure
         let ptr_to_a = &a;
         ptr_to_a->x = 5; // should fail, ptr_to_a is pointer to immutable, fail
         (*ptr_to_a).x = 5 // *ptr_to_a is evaluated to be of const struct A type

         // then check if pointer
         // but
         let mut_ptr_to_a = ptr_to_a as raw *mut A;
         mut_ptr_to_a->x = 5; // will fail again because of dot access rules above!

         */
    }

    fn check_assignment_is_place_expr(
        &self,
        lhs: &Expression,
        context: &mut SecondSemanticsPassContext
    )
    {
        match lhs {
            Expression::DotAccess(_) | Expression::ArrowAccess(_) |
            Expression::ArraySlice(_) | Expression::Unary(_) |
            Expression::Identifier(_) => {
                self.visit_expression(lhs, context);
                let lhs_type = context.type_validation.get_node_type(lhs);
                
                if !lhs_type.mutable {
                    panic!("Can't assign to non mutable type {:?}", lhs);
                }
                
                context.type_validation.set_node_type(
                    lhs,
                    lhs_type.clone()
                );
            },
            _ => {
                panic!(
                    "can't assign to non-lvalue expression {:?}",
                    lhs
                );
            }
        }
    }
}

impl TypeValidationContext {
    fn get_node_type<T: AstNode>(&self, node: &T) -> &Type {
        dev_assert!(
            self.evaluated_types.contains_key(&node),
            "No type defined for node {}", node.get_node_index()
        );

        &self.evaluated_types[&node.get_node_id()]
    }

    fn set_node_type<T: AstNode>(&mut self, node: &T, node_type: Type) {
        dev_assert!(
            !self.evaluated_types.contains_key(&node.get_node_id()),
            "type {:?} is already defined for node {}", node_type, node.get_node_id()
        );
        self.evaluated_types.insert(node.get_node_id(),  node_type);
    }

    fn builtin_type_from_type_kind(
        &self,
        referrer: AstNodeIndex,
        type_kind: &TypeKind,
        context: &SecondSemanticsPassContext,
    ) -> BuiltinType {
        match type_kind {
            TypeKind::Simple(simple_type) => {
                BuiltinType::from_simple_type(simple_type)
                    .unwrap_or_else(|| BuiltinType::Struct(
                        Box::new(
                            context.find_struct_type_from_name(
                                &referrer, &simple_type.name.lexeme
                            ).clone()
                        )
                    ))
            },
            TypeKind::Pointer(pointer_type) => {
                let inner_type= self.builtin_type_from_type_kind(
                    referrer,
                    &pointer_type.inner_type,
                    context
                );

                BuiltinType::Pointer(Box::new(PointerType::new(
                    inner_type,
                    pointer_type.points_to_mut
                )))
            },
        }
    }

    fn type_from_type_annotation(
        &self,
        type_annotation: &TypeAnnotation,
        context: &SecondSemanticsPassContext,
    ) -> Type {
        let builtin_type = self.builtin_type_from_type_kind(
            type_annotation.get_node_id(),
            &type_annotation.kind,
            context,
        );

        Type::new(builtin_type, type_annotation.is_mut)
    }
}

impl SecondSemanticsPassContext {
    fn find_struct_type_from_name<T: AstNode>(
        &self, node: &T, name: &String
    ) -> &StructType {
        let scope_index = self.name_scoping
            .local_scopes
            .get_reverse_scope_index(&node.get_node_id()).unwrap();
        let (_, scope) = self.name_scoping.local_scopes.find_from_scope(
            *scope_index, name
        ).unwrap();
        let struct_node_id = scope.find_struct_in_scope(
            &name
        ).unwrap();

        let evaluated_struct_type = self.type_validation.get_node_type(struct_node_id);
        let BuiltinType::Struct(struct_type) = &evaluated_struct_type.ttype else {
            panic!(
                "struct initializer type is not struct {}; got {}",
                name, evaluated_struct_type
            );
        };

        struct_type
    }
}

impl Semantics<SecondSemanticsPassContext> for TypeValidationSemantics {
    // fn visit_function_statement(&self, fn_statement: &FnStatement, function: &Function, context: &mut SecondSemanticsPassContext) {
    //     // function.arguments.iter().for_each(|arg| {
    //     //     arg.declared_type
    //     // });
    //     let return_exprs: Vec<_> = function.body
    //         .iter()
    //         .filter_map(
    //             |s| if let Statement::ReturnStatement(r) = s {
    //                 Some(&r.expression)
    //             } else {
    //                 None
    //             }
    //         ).collect();
    //
    //     if function.return_type.is_some() && return_exprs.is_empty() {
    //         panic!("Function body doesn't have return statement {:?}", function)
    //     }
    //
    //     for return_expr in return_exprs.iter() {
    //         match return_expr {
    //             Some(expr) => {
    //                 if function.return_type.is_none() {
    //                     panic!("void function can't have return statements with expression {:?}", return_expr);
    //                 }
    //
    //                 let expr_return_type = context.type_validation.evaluated_types[expr.get_node_id()];
    //                 if function_return_type != expr_return_type {
    //                     panic!(
    //                         "incompatible return type for function: function return type {}, but got {}",
    //                         function_return_type, expr_return_type
    //                     );
    //                 }
    //             },
    //             None => {
    //                 if function.return_type.is_some() {
    //                     panic!("Can't return void from function with declared return type {:?}", function.return_type.as_ref().unwrap());
    //                 }
    //             }
    //         }
    //     }
    // }
    //
    // fn visit_method_statement(&self, fn_statement: &FnStatement, method: &Method, context: &mut SecondSemanticsPassContext) {
    //     let return_exprs: Vec<_> = method.body
    //         .iter()
    //         .filter_map(
    //             |s| if let Statement::ReturnStatement(r) = s {
    //                 Some(&r.expression)
    //             } else {
    //                 None
    //             }
    //         ).collect();
    //
    //     if method.return_type.is_some() && return_exprs.is_empty() {
    //         panic!("Function body doesn't have return statement {:?}", method)
    //     }
    //
    //     for return_expr in return_exprs.iter() {
    //         match return_expr {
    //             Some(expr) => {
    //                 if method.return_type.is_none() {
    //                     panic!("void function can't have return statements with expression {:?}", return_expr);
    //                 }
    //
    //                 let expr_return_type = context.type_validation.get_node_type(
    //                     expr.as_ref()
    //                 );
    //                 if expr_return_type != expr_return_type {
    //                     panic!(
    //                         "incompatible return type for function: function return type {}, but got {}",
    //                         expr_return_type, expr_return_type
    //                     );
    //                 }
    //             },
    //             None => {
    //                 if method.return_type.is_some() {
    //                     panic!("Can't return void from function with declared return type {:?}", method.return_type.as_ref().unwrap());
    //                 }
    //             }
    //         }
    //     }
    // }

    fn visit_grouping(&self, grouping: &Grouping, context: &mut SecondSemanticsPassContext) {
        self.visit_grouping_default(grouping, context);

        context.type_validation.set_node_type(
            grouping,
            context.type_validation.get_node_type(
                grouping.expression.as_ref()
            ).clone()
        )
    }
    
    fn visit_struct_statement(
        &self,
        struct_statement: &StructStatement,
        context: &mut SecondSemanticsPassContext
    ) {
        self.visit_struct_statement_default(struct_statement, context);

        let typed_fields: HashMap<_, _> = struct_statement.fields.iter()
            .map(
                |fd| (
                    fd.name.lexeme.clone(),
                    context.type_validation.get_node_type(&fd.declared_type).clone()
                )
            )
            .collect();

        let struct_type = StructType {
            ast_node_index: struct_statement.get_node_id(),
            name: struct_statement.name.lexeme.clone(),
            fields: typed_fields
        };

        context.type_validation.set_node_type(
            struct_statement,
            Type::new(
                BuiltinType::Struct(Box::new(struct_type)),
                false,
            )
        );
    }

    fn visit_struct_initializer(
        &self,
        struct_initializer: &StructInitializer,
        context: &mut SecondSemanticsPassContext
    ) {
        self.visit_struct_initializer_default(struct_initializer, context);

        let struct_type = context.find_struct_type_from_name(
            struct_initializer, &struct_initializer.struct_name.lexeme
        );

        for (initialized_field, initializer) in &struct_initializer.field_initializers {
            let field_type = struct_type.fields.get(
                &initialized_field.lexeme
            ).unwrap();

            let initializer_type = context.type_validation.get_node_type(initializer);

            if !field_type.inner_type_eq(initializer_type) {
                panic!(
                    "mismatched types for field {} in struct initializer {}: expected {} but got {}",
                    initialized_field, struct_initializer.token,
                    field_type, initializer_type
                );
            }
        }

        context.type_validation.set_node_type(
            struct_initializer,
            Type::new(BuiltinType::Struct(Box::new(struct_type.clone())), false)
        );
    }

    fn visit_dot_access(&self, dot_access: &DotAccess, context: &mut SecondSemanticsPassContext) {
        self.visit_dot_access_default(dot_access, context);

        let object = context.type_validation.get_node_type(
            dot_access.object.as_ref()
        );

        let BuiltinType::Struct(struct_type) = &object.ttype else {
            panic!(
                "can't apply {} operator to non struct type {}",
                dot_access.name, object
            );
        };

        context.type_validation.set_node_type(
            dot_access,
            struct_type.fields[&dot_access.name.lexeme].clone()
        );
    }

    fn visit_arrow_access(
        &self,
        arrow_access: &ArrowAccess,
        context: &mut SecondSemanticsPassContext
    ) {
        self.visit_arrow_access_default(arrow_access, context);
        let pointer_object = context.type_validation.get_node_type(
            arrow_access.pointer.as_ref()
        );


        let BuiltinType::Pointer(pointer) = &pointer_object.ttype
        else {
            panic!(
                "can't apply -> operator {} to non-pointer type {}",
                arrow_access.name, pointer_object
            );
        };

        let BuiltinType::Struct(struct_type) = &pointer.inner_type else {
            panic!(
                "can't apply -> {} operator to non struct type {}",
                arrow_access.name, pointer.inner_type
            );
        };

        context.type_validation.set_node_type(
            arrow_access,
            struct_type.fields[&arrow_access.name.lexeme].clone()
        );
    }

    fn visit_call(&self, call: &Call, context: &mut SecondSemanticsPassContext) {
        self.visit_call_default(call, context);
        let callee_type = context.type_validation.get_node_type(call.callee.as_ref());

        let BuiltinType::Function(callee_function_type) = &callee_type.ttype
        else {
            panic!(
                "can't apply call operator to non function type {}; at {}",
                callee_type, call.token
            );
        };

        if callee_function_type.arguments.len() != call.arguments.len() {
            panic!(
                "function call expects {} parameters, but got {}",
                callee_function_type.arguments.len(), call.arguments.len()
            );
        }
        let arguments_types: Vec<_> = call.arguments
            .clone()
            .iter()
            .map(|arg|context.type_validation.get_node_type(arg))
            .collect();
        let mut args_mapping = callee_function_type
            .arguments
            .iter()
            .zip(arguments_types.clone());
        if args_mapping.any(|(expected, got)| expected != &got.ttype) {
            panic!(
                "incorrect argument types:\nexpected: {:?}\ngot: {:?}",
                callee_function_type.arguments, arguments_types
            );
        }
    }

     fn visit_literal(&self, literal: &LiteralNode, context: &mut SecondSemanticsPassContext) {
        let literal_type = Type::from_literal(&literal.literal);

        context.type_validation.set_node_type(
            literal,
            literal_type
        );
    }

    fn visit_identifier(
        &self,
        identifier: &Identifier,
        context: &mut SecondSemanticsPassContext
    ) {
        let Some(scope_index) = context.name_scoping.local_scopes.get_reverse_scope_index(
            identifier
        ) else {
            panic!("No scope found for identifier {}", identifier.name);
        };
        let Some((scope_index, scope)) = context.name_scoping
            .local_scopes
            .find_from_scope(
                *scope_index, &identifier.name.lexeme
            ) else {
            panic!(
                "Identifier is not found from current scope {} -> {}",
                scope_index, identifier.name
            );
        };

        let reference_node_id = scope
            .find_in_scope(&identifier.name.lexeme)
            .unwrap();
        let actual_node = context.parser.ast_nodes.get(&reference_node_id).unwrap();
        let identifier_type = context.type_validation.get_node_type(&reference_node_id);

        context.type_validation.set_node_type(identifier, identifier_type.clone());
    }

    fn visit_type_annotation(
        &self,
        type_annotation: &TypeAnnotation,
        context: &mut SecondSemanticsPassContext
    ) {
        let mut annotated_type = context.type_validation.type_from_type_annotation(
            type_annotation, context
        );

        annotated_type.mutable = type_annotation.is_mut;
        context.type_validation.set_node_type(
            type_annotation,
            annotated_type
        );
    }

    fn visit_typed_declaration(
        &self,
        type_declaration: &TypedDeclaration,
        context: &mut SecondSemanticsPassContext
    ) {
        self.visit_typed_declaration_default(type_declaration, context);

        context.type_validation.set_node_type(
            type_declaration,
            context.type_validation.get_node_type(
                &type_declaration.declared_type
            ).clone()
        );
    }

    fn visit_let_statement(
        &self,
        let_statement: &LetStatement,
        context: &mut SecondSemanticsPassContext
    ) {
        self.visit_let_statement_default(let_statement, context);
        let (annotated, initialized) = (
            let_statement.variable_type.is_some(),
            let_statement.initializer.is_some(),
        );

        let let_statement_type = match (annotated, initialized) {
            (true, true) => {
                let annotated_type = context.type_validation.get_node_type(
                    let_statement.variable_type.as_ref().unwrap()
                );
                let initialized_type = context.type_validation.get_node_type(
                    let_statement.initializer.as_ref().unwrap().as_ref()
                );

                if !annotated_type.inner_type_eq(&initialized_type) {
                    panic!(
                        "type mismatch: expected {} but got {}",
                        annotated_type, initialized_type
                    )
                }

                Type::new(annotated_type.ttype.clone(), let_statement.is_mut)
            },
            (true, false) => {
                // TODO!
                // if !context.scope_resolving.is_variable_initialized() {
                //     panic!(
                //         "variable is not initialized {}",
                //         let_statement.name
                //     );
                // }
                let annotated_type = context.type_validation.get_node_type(
                    let_statement.variable_type.as_ref().unwrap()
                );

                Type::new(annotated_type.ttype.clone(), let_statement.is_mut)
            },
            (false, true) => {
                let initialized_type = context.type_validation.get_node_type(
                    let_statement.initializer.as_ref().unwrap().as_ref()
                );

                Type::new(initialized_type.ttype.clone(), let_statement.is_mut)
            },
            _ => panic!(
                "Let statement {} has to be either annotated or initialized",
                let_statement.name
            ),
        };

        context.type_validation.set_node_type(
            let_statement,
            let_statement_type
        );
    }
    
    fn visit_unary(&self, unary: &Unary, context: &mut SecondSemanticsPassContext) {
        self.visit_unary_default(unary, context);
        let right_type = context.type_validation.get_node_type(
            unary.expression.as_ref()
        );

        let Some(result_type) = match_unary_op(
            unary.operator.token_type, right_type
        ) else {
            panic!(
                "Can't apply {} operator to type {}",
                unary.operator, right_type
            );
        };

        context.type_validation.set_node_type(unary, result_type);
    }

    fn visit_cast(&self, cast: &Cast, context: &mut SecondSemanticsPassContext) {
        self.visit_cast_default(cast, context);

        let target_type = context.type_validation.get_node_type(
            &cast.target_type
        );
    
        let result_type = if !cast.is_reinterpret_cast {
            let left_type = context.type_validation.get_node_type(
                cast.left.as_ref()
            );

            if !verify_cast_operator(left_type, target_type) {
                panic!(
                    "Cannot cast {} to {} in {}",
                    left_type, target_type, cast.token
                );
            }
    
            target_type
        } else {
            target_type
        };

        context.type_validation.set_node_type(cast, result_type.clone());
    }

    fn visit_binary(&self, binary: &Binary, context: &mut SecondSemanticsPassContext) {
        self.visit_binary_default(binary, context);

        let left_type = context.type_validation.get_node_type(
            binary.left.as_ref()
        );
        let right_type = context.type_validation.get_node_type(
            binary.right.as_ref()
        );

        let Some(result_type) = match_binary_op(
            binary.operator.token_type, left_type, right_type
        ) else {
            panic!(
                "Incompatible type pair for {:?} binary operator ({:?} {:?} {:?})",
                binary.operator.token_type,
                left_type, binary.operator.token_type,
                right_type
            );
        };
        context.type_validation.evaluated_types.insert(binary.node_id, result_type);
    }

    fn visit_inplace_assignment(
        &self,
        inplace_assignment: &InplaceAssignment,
        context: &mut SecondSemanticsPassContext
    ) {
        self.check_assignment_is_place_expr(&inplace_assignment.lhs, context);
        // self.check_assignment_to_mut(&assignment.lhs, context);
        self.visit_expression(&inplace_assignment.rhs, context);
    
        let left_type = context.type_validation.get_node_type(
            inplace_assignment.lhs.as_ref()
        );
        let right_type = context.type_validation.get_node_type(
            inplace_assignment.rhs.as_ref()
        );
    
        let Some(assignment_type) = match_inplace_assignment_op(
            inplace_assignment.operator.token_type, &left_type, &right_type
        ) else {
            panic!(
                "Can't in-place assign {} to {} at {}",
                left_type,
                right_type,
                inplace_assignment.token
            );
        };
    
        context.type_validation.set_node_type(inplace_assignment, assignment_type);
    }

    fn visit_assignment(&self, assignment: &Assignment, context: &mut SecondSemanticsPassContext) {
        self.check_assignment_is_place_expr(&assignment.lhs, context);
        // self.check_assignment_to_mut(&assignment.lhs, context);
        self.visit_expression(&assignment.rhs, context);

        let left_type = context.type_validation.get_node_type(
            assignment.lhs.as_ref()
        );
        let right_type = context.type_validation.get_node_type(
            assignment.rhs.as_ref()
        );
        if !left_type.inner_type_eq(right_type) {
            panic!(
                "at {}: cannot assign {} type to {} types",
                assignment.token,
                left_type, right_type
            );
        }

        context.type_validation.set_node_type(assignment, left_type.clone());
    }
}