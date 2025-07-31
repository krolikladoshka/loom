use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::parser::semantics::FirstSemanticsPassContext;
use crate::syntax::ast::{AstNodeIndex, BlockExpression, ConstStatement, FnStatement, Function, IfElseExpression, IfElseStatement, ImplStatement, LetStatement, Method, Statement, StaticStatement, StructStatement, Type, WhileStatement};
use std::collections::HashMap;
use crate::dev_assert_ne;


pub type DeclarationMap<T> = HashMap<String, T>;
pub type ScopeMap<T> = HashMap<String, T>;

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {}

#[derive(Debug, Clone)]
pub struct MethodDeclaration {}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub bound_struct: String,
    pub ast_node_index: AstNodeIndex,
    pub struct_ast_node_index: AstNodeIndex,
    pub static_variables: DeclarationMap<AstNodeIndex>,
    pub const_variables: DeclarationMap<AstNodeIndex>,
    pub functions: DeclarationMap<FunctionDeclaration>,
    pub methods: DeclarationMap<MethodDeclaration>,
}

impl ImplBlock {
    pub fn new(
        bound_struct: String,
        ast_node_index: AstNodeIndex,
        struct_ast_node_index: AstNodeIndex,
    ) -> Self {
        Self {
            bound_struct,
            ast_node_index,
            struct_ast_node_index,
            static_variables: Default::default(),
            const_variables: Default::default(),
            functions: Default::default(),
            methods: Default::default(),
        }
    }

    #[inline(always)]
    pub fn contains_function(&self, name: &str) -> bool {
        self.functions.contains_key(name) || self.methods.contains_key(name)
    }

    #[inline(always)]
    pub fn contains_name(&self, name: &str) -> bool {
        self.static_variables.contains_key(name) ||
            self.const_variables.contains_key(name) ||
            self.contains_function(name)
    }
}


#[derive(Debug, Clone)]
pub struct StructDeclaration {
    name: String,
    ast_node_index: AstNodeIndex,
    fields: HashMap<String, Type>,
}

impl StructDeclaration {
    pub fn new(name: String, ast_node_index: AstNodeIndex) -> Self {
        Self {
            name,
            ast_node_index,
            fields: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub parent_scope: Option<usize>,
    pub static_declarations: ScopeMap<AstNodeIndex>,
    pub const_declarations: ScopeMap<AstNodeIndex>,
    pub functions: ScopeMap<AstNodeIndex>,
    pub structs: ScopeMap<StructDeclaration>,
    pub variables: ScopeMap<Vec<AstNodeIndex>>,
    pub impl_blocks: ScopeMap<ImplBlock>,
}

impl Scope {
    pub fn find_in_scope(&self, name: &str) -> bool {
        self.variables.contains_key(name) ||
            self.functions.contains_key(name) ||
            self.structs.contains_key(name) ||
            self.static_declarations.contains_key(name) ||
            self.const_declarations.contains_key(name)
    }

    pub fn find_in_impl_block(&self, name: &str, struct_name: &str) -> Option<&ImplBlock> {
        let impl_block = self.impl_blocks.get(struct_name)?;

        if impl_block.contains_name(name) {
            Some(impl_block)
        } else {
            None
        }
    }

    pub fn find_in_impl_block_mut(
        &mut self,
        name: &str,
        struct_name: &str
    ) -> Option<&mut ImplBlock>
    {
        let impl_block = self.impl_blocks.get_mut(struct_name)?;

        if impl_block.contains_name(name) {
            Some(impl_block)
        } else {
            None
        }
    }
}



#[derive(Debug, Clone, Default)]
pub struct ScopeTree {
    tree: Vec<usize>,
    scopes: Vec<Scope>,
}

impl ScopeTree {
    pub fn new() -> Self {
        Self {
            tree: vec![0],
            scopes: vec![Scope::default()],
        }
    }
    fn find_in_all_impl_blocks(
        &self,
        name: &str,
        struct_name: &str,
    ) -> bool {
        self.scopes
            .iter()
            .filter(|scope| !scope.impl_blocks.is_empty())
            .any(|scope| scope.find_in_impl_block(name, struct_name).is_some())
    }
    
    pub fn stack_scope(&mut self, parent: Option<usize>) -> (usize, &mut Scope) {
        let parent_scope = parent.unwrap_or(0);
        self.scopes.push(Scope::default());
        self.tree.push(parent_scope);

        let idx = self.scopes.len() - 1;
        (idx, &mut self.scopes[idx])
    }

    #[inline(always)]
    pub fn scope_contains(&self, scope_index: usize, name: &str) ->  bool {
        let Some(scope) = self.scopes.get(scope_index) else {
            return false;
        };

        scope.find_in_scope(name)
    }

    pub fn find_scope_index(&self, start_scope: usize, name: &str) -> Option<usize> {
        if self.scope_contains(start_scope, name) {
            return Some(start_scope);
        }

        if start_scope == 0 {
            return None;
        }

        let parent_scope = self.tree[start_scope];

        self.find_scope_index(parent_scope, name)
    }

    #[inline(always)]
    pub fn find_in_scope(&mut self, scope_index: usize, name: &str) -> Option<&mut Scope> {
        if self.scope_contains(scope_index, name) {
            self.scopes.get_mut(scope_index)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn find_in_current_scope(&mut self, name: &str) -> Option<&mut Scope> {
        self.find_in_scope(self.tree.len() - 1, name)
    }

    pub fn find_from_scope(&mut self, scope_index: usize, name: &str) -> Option<(usize, &mut Scope)> {
        if scope_index >= self.scopes.len() {
            return None;
        }
        let index = self.find_scope_index(scope_index, name)?;

        Some((index, self.scopes.get_mut(index).unwrap()))
    }

    #[inline(always)]
    pub fn find_from_current_scope(&mut self, name: &str) -> Option<(usize, &mut Scope)> {
        self.find_from_scope(self.tree.len() - 1, name)
    }
}

#[derive(Debug, Clone)]
struct StructImplView {
    struct_name: String,
    struct_node_index: AstNodeIndex,
    struct_scope_index: usize,
    impl_block_node_index: AstNodeIndex,
    impl_block_scope_index: usize,
}

impl StructImplView {
    pub fn new(
        struct_name: String,
        struct_node_index: AstNodeIndex,
        struct_scope_index: usize,
        impl_block_node_index: AstNodeIndex,
        impl_block_scope_index: usize,
    ) -> Self {
        Self {
            struct_name,
            struct_node_index,
            struct_scope_index,
            impl_block_node_index,
            impl_block_scope_index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NameResolvingContext {
    pub local_scopes: ScopeTree,

    current_scope: usize,
    struct_impl_view: Vec<StructImplView>,
    is_within_impl_block: bool,
}

impl Default for NameResolvingContext {
    fn default() -> Self {
        Self {
            local_scopes: ScopeTree::new(),
            current_scope: 0,
            struct_impl_view: vec![],
            is_within_impl_block: false,
        }
    }
}

impl NameResolvingContext {
    #[inline(always)]
    pub fn get_scope(&self, scope_index: usize) -> &Scope {
        self.local_scopes.scopes.get(scope_index).unwrap()
    }

    #[inline(always)]
    pub fn get_scope_mut(&mut self, scope_index: usize) -> &mut Scope {
        self.local_scopes.scopes.get_mut(scope_index).unwrap()
    }

    #[inline(always)]
    pub fn get_current_scope(&self) -> &Scope {
        self.get_scope(self.current_scope)
    }

    #[inline(always)]
    pub fn get_current_scope_mut(&mut self) -> &mut Scope {
        self.get_scope_mut(self.current_scope)
    }
}

impl FirstSemanticsPassContext {
    #[inline(always)]
    pub fn with_new_naming_scope<Block>(&mut self, block: Block)
    where
        Block: FnOnce(&mut Self)
    {
        let prev_scope_id = self.name_resolving.current_scope;
        let (new_scope_id, new_scope) = self.name_resolving.local_scopes.stack_scope(
            Some(prev_scope_id),
        );
        self.name_resolving.current_scope = new_scope_id;

        block(self);

        self.name_resolving.current_scope = prev_scope_id;
    }
}

pub struct NameResolvingSemantics;

impl AstContext for NameResolvingContext { }

impl NameResolvingSemantics {
    fn visit_function_statement_within_new_scope(
        &self,
        fn_statement: &FnStatement,
        function: &Function,
        context: &mut FirstSemanticsPassContext
    ) {
        context.with_new_naming_scope(|context| {
            let function_scope = context.name_resolving.get_current_scope_mut();

            for arg in function.arguments.iter() {
                if function_scope.variables.contains_key(&arg.name.lexeme) {
                    panic!(
                        "Parameter {} is already defined for function {}",
                        arg.name.lexeme,
                        function.name.lexeme
                    );
                }
                function_scope.variables.entry(arg.name.lexeme.clone()).or_default().push(
                    AstNodeIndex(1) // TODO:
                )
            }

            self.visit_function_statement_default(fn_statement, function, context);
        });
    }
}

impl Semantics<FirstSemanticsPassContext> for NameResolvingSemantics {
    fn visit_statement(&self, statement: &Statement, context: &mut FirstSemanticsPassContext) {
        self.visit_statement_default(statement, context)
    }

    fn visit_let_statement(
        &self,
        let_statement: &LetStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        dev_assert_ne!(context.current_scope, 0);
        self.visit_let_statement_default(let_statement, context);

        let scope = context.name_resolving.get_current_scope_mut();

        let variable_name = &let_statement.name.lexeme;
        if scope.const_declarations.contains_key(variable_name) {
            panic!(
                "Let variable {} can't shadow const variable in the same scope",
                variable_name
            );
        }
        if scope.static_declarations.contains_key(variable_name) {
            panic!(
                "let variable {} can't shadow static variable in the same scope",
                variable_name
            );
        }

        scope
            .variables
            .entry(let_statement.name.lexeme.clone())
            .or_default()
            .push(let_statement.node_id);
    }

    fn visit_static_statement(
        &self,
        static_statement: &StaticStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        let scope = context.name_resolving.get_current_scope_mut();

        if scope.find_in_scope(&static_statement.name.lexeme) {
            panic!("can't redefine static variables in same scope");
        }

        scope.static_declarations.insert(static_statement.name.lexeme.clone(), static_statement.node_id);

        self.visit_expression(&static_statement.initializer, context);
    }

    fn visit_const_statement(
        &self,
        const_statement: &ConstStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        let scope = context.name_resolving.get_current_scope_mut();

        if scope.find_in_scope(&const_statement.name.lexeme) {
            panic!("can't redefine const variables in same scope");
        }

        scope.const_declarations.insert(
            const_statement.name.lexeme.clone(),
            const_statement.node_id
        );

        self.visit_expression(&const_statement.initializer, context);
    }

    fn visit_while_statement(&self, while_statement: &WhileStatement, context: &mut FirstSemanticsPassContext) {
        context.with_new_naming_scope(|context|
            self.visit_while_statement_default(while_statement, context)
        );
    }

    fn visit_function_statement(
        &self,
        fn_statement: &FnStatement,
        function: &Function,
        context: &mut FirstSemanticsPassContext
    ) {
        if context.name_resolving.is_within_impl_block {
            let bound_struct_impl_view = context
                .name_resolving
                .struct_impl_view
                .last()
                .unwrap();
            {
                let struct_scope = context
                    .name_resolving
                    .local_scopes
                    .scopes
                    .get_mut(bound_struct_impl_view.struct_scope_index)
                    .unwrap();


                let Some(_) = struct_scope
                    .structs
                    .get_mut(&bound_struct_impl_view.struct_name)
                else {
                    panic!(
                        "Struct {} was not found for function {}",
                        bound_struct_impl_view.struct_name, bound_struct_impl_view.struct_name
                    );
                };
            }

            if context.name_resolving.local_scopes.find_in_all_impl_blocks(
                &function.name.lexeme,
                &bound_struct_impl_view.struct_name
            ) {
                panic!(
                    "Duplicated function name declaration {} in impl {} block",
                    function.name, bound_struct_impl_view.struct_name
                );
            }
            
            let impl_block = context
                .name_resolving
                .local_scopes
                .scopes
                .get_mut(bound_struct_impl_view.impl_block_scope_index)
                .unwrap()
                .impl_blocks
                .get_mut(&bound_struct_impl_view.struct_name)
                .unwrap();
            
            impl_block.functions.insert(
                function.name.lexeme.clone(), FunctionDeclaration {}
            );
            
            let prev_scope = context.name_resolving.current_scope;
            context.name_resolving.current_scope = bound_struct_impl_view.impl_block_scope_index;
            
            self.visit_function_statement_within_new_scope(
                fn_statement,
                function,
                context
            );
            
            context.name_resolving.current_scope = prev_scope;
        } else {
            let scope = context.name_resolving.get_current_scope_mut();
            
            if scope.find_in_scope(&function.name.lexeme) {
                panic!(
                    "Duplicated function declaration {}",
                    function.name.lexeme
                );
            }
            scope.functions.insert(
                function.name.lexeme.clone(),
                fn_statement.node_id,
            );
            
            self.visit_function_statement_within_new_scope(
                fn_statement,
                function,
                context
            );
        }
    }

    fn visit_method_statement(
        &self,
        fn_statement: &FnStatement,
        method: &Method,
        context: &mut FirstSemanticsPassContext
    ) {
        let bound_struct_impl_view = context
            .name_resolving
            .struct_impl_view
            .last()
            .unwrap();
        {
            let struct_scope = context
                .name_resolving
                .local_scopes
                .scopes
                .get_mut(bound_struct_impl_view.struct_scope_index)
                .unwrap();


            let Some(struct_declaration) = struct_scope
                .structs
                .get_mut(&bound_struct_impl_view.struct_name)
            else {
                panic!(
                    "Struct {} was not found for function {}",
                    bound_struct_impl_view.struct_name, bound_struct_impl_view.struct_name
                );
            };
        }
        if context.name_resolving.local_scopes.find_in_all_impl_blocks(
            &method.name.lexeme,
            &bound_struct_impl_view.struct_name
        ) {
            panic!(
                "Duplicated method name declaration {} in impl {} block",
                method.name, bound_struct_impl_view.struct_name
            );
        }
        
        let impl_block = context
            .name_resolving
            .local_scopes
            .scopes
            .get_mut(bound_struct_impl_view.impl_block_scope_index)
            .unwrap()
            .impl_blocks
            .get_mut(&bound_struct_impl_view.struct_name)
            .unwrap();

        impl_block.methods.insert(
            method.name.lexeme.clone(), MethodDeclaration {}
        );

        let prev_scope = context.name_resolving.current_scope;
        context.name_resolving.current_scope = bound_struct_impl_view.impl_block_scope_index;
        
        context.with_new_naming_scope(
            |context| {
                let prev_impl_block_flag = context.name_resolving.is_within_impl_block;
                context.name_resolving.is_within_impl_block = true;
                
                let function_scope = context.name_resolving.get_current_scope_mut();
                function_scope.variables.entry("self".to_string()).or_default().push(
                    AstNodeIndex(1) // TODO:
                );

                for arg in method.arguments.iter() {
                    if function_scope.variables.contains_key(&arg.name.lexeme) {
                        panic!(
                            "Parameter {} is already defined for method {}",
                            arg.name,
                            method.name.lexeme
                        );
                    }
                    function_scope.variables.entry(arg.name.lexeme.clone()).or_default().push(
                        AstNodeIndex(1) // TODO:
                    );
                }

                self.visit_all_statements(&method.body, context);
                context.name_resolving.is_within_impl_block = prev_impl_block_flag;
            }
        );

        context.name_resolving.current_scope = prev_scope;
    }

    fn visit_struct_statement(
        &self,
        struct_statement: &StructStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        let scope = context.name_resolving.get_current_scope_mut();

        if scope.find_in_scope(&struct_statement.name.lexeme) {
            panic!("Struct {} is already defined in this scope", struct_statement.name.lexeme);
        }

        // TODO: define struct names for further type validation
        scope.structs.insert(struct_statement.name.lexeme.clone(), StructDeclaration::new(
            struct_statement.name.lexeme.clone(),
            struct_statement.node_id
        ));
    }

    fn visit_impl_statement(
        &self,
        impl_statement: &ImplStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        let struct_name = &impl_statement.implemented_type;
        let (struct_scope_idx, struct_ast_node_index) = {
            let Some((struct_scope_idx, struct_scope)) = context
                .name_resolving
                .local_scopes
                .find_from_current_scope(
                    &struct_name.lexeme
                )

            else {
                panic!(
                    "Name {} was not found in scope for method {}",
                    struct_name.lexeme, struct_name.lexeme
                );
            };

            let Some(struct_declaration) = struct_scope
                .structs
                .get_mut(&struct_name.lexeme)
            else {
                panic!(
                    "Struct {} was not found in scope for method {}",
                    struct_name.lexeme, struct_name.lexeme
                );
            };

            (struct_scope_idx, struct_declaration.ast_node_index)
        };
        

        let current_scope = context.name_resolving.get_current_scope_mut();
        current_scope
            .impl_blocks
            .entry(struct_name.lexeme.clone())
            .or_insert(ImplBlock::new(
                impl_statement.implemented_type.lexeme.clone(),
                impl_statement.node_id,
                struct_ast_node_index,
            ));

        context.name_resolving.struct_impl_view.push(StructImplView::new(
            struct_name.lexeme.clone(),
            struct_ast_node_index,
            struct_scope_idx,
            impl_statement.node_id,
            context.name_resolving.current_scope,
        ));

        let prev_impl_block_flag = context.name_resolving.is_within_impl_block;
        context.name_resolving.is_within_impl_block = true;
        // context.with_new_naming_scope(|context|
            self.visit_impl_statement_default(impl_statement, context);
        // );
        context.name_resolving.is_within_impl_block = prev_impl_block_flag
    }

    fn visit_if_else_statement(&self, if_else_statement: &IfElseStatement, context: &mut FirstSemanticsPassContext) {
        self.visit_expression(&if_else_statement.condition, context);

        context.with_new_naming_scope(|context|
            self.visit_all_statements(&if_else_statement.then_branch, context)
        );

        if  let Some(else_branch) = &if_else_statement.else_branch {
            context.with_new_naming_scope(|context|
                self.visit_all_statements(else_branch, context)
            );
        }
    }

    fn visit_if_else(&self, if_else: &IfElseExpression, context: &mut FirstSemanticsPassContext) {
        self.visit_if_else_default(if_else, context)
    }

    fn visit_block_expression(&self, block: &BlockExpression, context: &mut FirstSemanticsPassContext) {
        context.with_new_naming_scope(|context|
            self.visit_block_expression_default(block, context)
        );
    }
}
