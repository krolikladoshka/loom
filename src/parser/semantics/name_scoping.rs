use crate::parser::semantics::traits::{AstContext, Semantics};
use crate::parser::semantics::{FirstSemanticsPassContext};
use crate::syntax::ast::{current_id, AstNode, AstNodeIndex, BlockExpression, ConstStatement, FnStatement, Function, IfElseExpression, IfElseStatement, ImplStatement, LetStatement, Method, Statement, StaticStatement, StructStatement, Type, WhileStatement};
use std::collections::HashMap;
use crate::{dev_assert, dev_assert_ne};
use crate::parser::semantics::name_scoping::ScopeError::NameIsAlreadyDefined;

pub type DeclarationMap<T> = HashMap<String, T>;
pub type ScopeMap<T> = HashMap<String, T>;

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub bound_struct: String,
    pub ast_node_index: AstNodeIndex,
    pub struct_ast_node_index: AstNodeIndex,
    pub static_variables: DeclarationMap<AstNodeIndex>,
    pub const_variables: DeclarationMap<AstNodeIndex>,
    pub functions: DeclarationMap<AstNodeIndex>,
    pub methods: DeclarationMap<AstNodeIndex>,
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
    pub structs: ScopeMap<AstNodeIndex>,
    pub variables: ScopeMap<Vec<AstNodeIndex>>,
    pub impl_blocks: ScopeMap<ImplBlock>,
}

impl Scope {
    pub fn new(parent_scope: Option<usize>) -> Self {
        let mut scope = Scope::default();
        scope.parent_scope = parent_scope;

        scope
    }

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
    pub tree: Vec<usize>,
    pub scopes: Vec<Scope>,
    pub scope_id_by_node_index: Vec<usize>
}

enum ScopeError {
    NameIsAlreadyDefined,
}
type ScopeResult = Result<(), ScopeError>;

impl ScopeTree {
    pub fn new() -> Self {
        let mut scopes_vec = Vec::with_capacity(
            current_id().0.ilog10() as usize
        );
        scopes_vec.push(Scope::default());

        Self {
            tree: vec![0],
            scopes: scopes_vec,
            scope_id_by_node_index: vec![0; current_id().into()],
        }
    }

    #[inline(always)]
    pub fn is_variable_defined(&self, scope_id: usize, name: &str) -> bool {
        dev_assert!(
            scope_id >= 0 && scope_id < self.scopes.len(),
            "Scope id does not exist {}", scope_id
        );

        self.scopes[scope_id].variables.contains_key(name)
    }

    pub fn add_variable(
        &mut self, scope_id: usize,
        variable_name: String,
        variable_node_id: AstNodeIndex
    ) {
        dev_assert!(
            scope_id >= 0 && scope_id < self.scopes.len(),
            "Scope id does not exist {}", scope_id
        );
        dev_assert!(
            variable.ast_node_index.0 >= 0 &&
            variable.ast_node_index < self.scope_id_by_node_index.len(),
            "variable ast node index is out of range of predefined map of scopes: {}",
            variable.ast_node_index
        );

        self.scopes[scope_id]
            .variables
            .entry(variable_name)
            .or_default()
            .push(variable_node_id);
        self.scope_id_by_node_index[variable_node_id.0] = scope_id;
    }

    pub fn add_const(&mut self, scope_id: usize, const_variable: &ConstStatement) -> ScopeResult {
        dev_assert!(
            scope_id >= 0 && scope_id < self.scopes.len(),
            "Scope id does not exist {}", scope_id
        );
        dev_assert!(
            variable.ast_node_index.0 >= 0 &&
            variable.ast_node_index < self.scope_id_by_node_index.len(),
            "const variable ast node index is out of range of predefined map of scopes: {}",
            variable.ast_node_index
        );

        self.check_if_already_defined(scope_id, &const_variable.name.lexeme)?;

        self.scopes[scope_id].const_declarations.insert(
            const_variable.name.lexeme.clone(), const_variable.node_id
        );
        self.scope_id_by_node_index[const_variable.node_id.0] = scope_id;

        Ok(())
    }

    pub fn add_static(
        &mut self,
        scope_id: usize,
        static_variable: &StaticStatement
    ) -> ScopeResult {
        dev_assert!(
            scope_id >= 0 && scope_id < self.scopes.len(),
            "Scope id does not exist {}", scope_id
        );
        dev_assert!(
            variable.ast_node_index.0 >= 0 &&
            variable.ast_node_index < self.scope_id_by_node_index.len(),
            "static_variable variable ast node index is out of range
            of predefined map of scopes: {}",
            variable.ast_node_index
        );

        self.check_if_already_defined(scope_id, &static_variable.name.lexeme)?;

        self.scopes[scope_id].const_declarations.insert(
            static_variable.name.lexeme.clone(), static_variable.node_id
        );
        self.scope_id_by_node_index[static_variable.node_id.0] = scope_id;

        Ok(())
    }

    fn add_function(
        &mut self,
        scope_id: usize,
        function: &Function,
        function_node_id: AstNodeIndex
    ) -> ScopeResult {
        dev_assert!(
            scope_id >= 0 && scope_id < self.scopes.len(),
            "Scope id does not exist {}", scope_id
        );

        self.check_if_already_defined(scope_id, &function.name.lexeme)?;

        self.scopes[scope_id].functions.insert(
            function.name.lexeme.clone(),
            function_node_id
        );
        self.scope_id_by_node_index[function_node_id.0] = scope_id;

        Ok(())
    }


    #[inline(always)]
    fn check_if_already_defined(&self, scope_id: usize, name: &str) -> ScopeResult {
        if self.scope_contains(scope_id, name) {
            return Err(ScopeError::NameIsAlreadyDefined);
        }

        Ok(())
    }

    pub fn find_in_all_impl_blocks(
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
        self.scopes.push(Scope::new(parent));
        self.tree.push(parent.unwrap_or(0));

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

    pub fn find_in_scope(&self, scope_index: usize, name: &str) -> Option<&Scope> {
        if self.scope_contains(scope_index, name) {
            self.scopes.get(scope_index)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn find_in_scope_mut(&mut self, scope_index: usize, name: &str) -> Option<&mut Scope> {
        if self.scope_contains(scope_index, name) {
            self.scopes.get_mut(scope_index)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn find_in_current_scope(&self, name: &str) -> Option<&Scope> {
        self.find_in_scope(self.tree.len() - 1, name)
    }

    #[inline(always)]
    pub fn find_in_current_scope_mut(&mut self, name: &str) -> Option<&mut Scope> {
        self.find_in_scope_mut(self.tree.len() - 1, name)
    }

    pub fn find_from_scope(&self, scope_index: usize, name: &str) -> Option<(usize, &Scope)> {
        if scope_index >= self.scopes.len() {
            return None;
        }
        let index = self.find_scope_index(scope_index, name)?;

        Some((index, self.scopes.get(index).unwrap()))
    }

    pub fn find_from_scope_mut(
        &mut self, scope_index: usize, name: &str
    ) -> Option<(usize, &mut Scope)> {
        if scope_index >= self.scopes.len() {
            return None;
        }
        let index = self.find_scope_index(scope_index, name)?;

        Some((index, self.scopes.get_mut(index).unwrap()))
    }

    #[inline(always)]
    pub fn find_from_current_scope(&mut self, name: &str) -> Option<(usize, &Scope)> {
        self.find_from_scope(self.tree.len() - 1, name)
    }

    #[inline(always)]
    pub fn find_from_current_scope_mut(
        &mut self, name: &str
    ) -> Option<(usize, &mut Scope)> {
        self.find_from_scope_mut(self.tree.len() - 1, name)
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
pub struct NameScopingContext {
    pub local_scopes: ScopeTree,
    current_scope: usize,
    struct_impl_view: Vec<StructImplView>,
    is_within_impl_block: bool,
}

impl Default for NameScopingContext {
    fn default() -> Self {
        Self {
            local_scopes: ScopeTree::new(),
            current_scope: 0,
            struct_impl_view: vec![],
            is_within_impl_block: false,
        }
    }
}

impl NameScopingContext {
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
        let prev_scope_id = self.name_scoping.current_scope;
        let (new_scope_id, _) = self.name_scoping.local_scopes.stack_scope(
            Some(prev_scope_id),
        );
        self.name_scoping.current_scope = new_scope_id;

        block(self);

        self.name_scoping.current_scope = prev_scope_id;
    }
}

pub struct NameScopingSemantics;

impl AstContext for NameScopingContext { }

impl NameScopingSemantics {
    fn visit_function_statement_within_new_scope(
        &self,
        fn_statement: &FnStatement,
        function: &Function,
        context: &mut FirstSemanticsPassContext
    ) {
        context.with_new_naming_scope(|context| {
            let function_scope = context.name_scoping.current_scope;
            let local_scopes = &mut context.name_scoping.local_scopes;

            for arg in function.arguments.iter() {
                if local_scopes.is_variable_defined(function_scope, &arg.name.lexeme) {
                    panic!(
                        "Parameter {} is already defined for function {}",
                        arg.name.lexeme,
                        function.name.lexeme
                    );
                }
                local_scopes.add_variable(
                    function_scope,
                    arg.name.lexeme.clone(),
                    arg.node_id
                );
            }

            self.visit_function_statement_default(fn_statement, function, context);
        });
    }
}

impl Semantics<FirstSemanticsPassContext> for NameScopingSemantics {
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
        let local_scopes = &mut context.name_scoping.local_scopes;
        // let scope = context.name_resolving.get_current_scope_mut();

        let variable_name = &let_statement.name.lexeme;
        if local_scopes.scope_contains(context.name_scoping.current_scope, variable_name) {
            panic!(
                "let variable {} can't shadow const or static variables in the same scope",
                variable_name
            );
        }

        local_scopes.add_variable(
            context.name_scoping.current_scope,
            variable_name.clone(),
            let_statement.node_id,
        );
    }

    fn visit_static_statement(
        &self,
        static_statement: &StaticStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        if let Err(NameIsAlreadyDefined) = context.name_scoping.local_scopes.add_static(
            context.name_scoping.current_scope,
            static_statement
        ) {
            panic!("can't redefine static variables in same scope");
        }

        self.visit_expression(&static_statement.initializer, context);
    }

    fn visit_const_statement(
        &self,
        const_statement: &ConstStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        if let Err(NameIsAlreadyDefined) = context.name_scoping.local_scopes.add_const(
            context.name_scoping.current_scope,
            const_statement,
        ) {
            panic!("can't redefine const variables in same scope");
        }

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
        if context.name_scoping.is_within_impl_block {
            let bound_struct_impl_view = context
                .name_scoping
                .struct_impl_view
                .last()
                .unwrap();
            {
                let struct_scope = context
                    .name_scoping
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

            if context.name_scoping.local_scopes.find_in_all_impl_blocks(
                &function.name.lexeme,
                &bound_struct_impl_view.struct_name
            ) {
                panic!(
                    "Duplicated function name declaration {} in impl {} block",
                    function.name, bound_struct_impl_view.struct_name
                );
            }
            
            let impl_block = context
                .name_scoping
                .local_scopes
                .scopes
                .get_mut(bound_struct_impl_view.impl_block_scope_index)
                .unwrap()
                .impl_blocks
                .get_mut(&bound_struct_impl_view.struct_name)
                .unwrap();
            
            impl_block.functions.insert(
                function.name.lexeme.clone(), fn_statement.node_id
            );
            
            let prev_scope = context.name_scoping.current_scope;
            context.name_scoping.current_scope = bound_struct_impl_view.impl_block_scope_index;
            
            self.visit_function_statement_within_new_scope(
                fn_statement,
                function,
                context
            );
            
            context.name_scoping.current_scope = prev_scope;
        } else {
            if let Err(NameIsAlreadyDefined) = context.name_scoping.local_scopes.add_function(
                context.name_scoping.current_scope,
                function,
                fn_statement.node_id,
            ) {
                panic!(
                    "Duplicated function declaration {}",
                    function.name.lexeme
                );
            }
            
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
            .name_scoping
            .struct_impl_view
            .last()
            .unwrap();
        {
            let struct_scope = context
                .name_scoping
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
        if context.name_scoping.local_scopes.find_in_all_impl_blocks(
            &method.name.lexeme,
            &bound_struct_impl_view.struct_name
        ) {
            panic!(
                "Duplicated method name declaration {} in impl {} block",
                method.name, bound_struct_impl_view.struct_name
            );
        }
        
        let impl_block = context
            .name_scoping
            .local_scopes
            .scopes
            .get_mut(bound_struct_impl_view.impl_block_scope_index)
            .unwrap()
            .impl_blocks
            .get_mut(&bound_struct_impl_view.struct_name)
            .unwrap();

        impl_block.methods.insert(
            method.name.lexeme.clone(), fn_statement.get_node_id()
        );

        let prev_scope = context.name_scoping.current_scope;
        context.name_scoping.current_scope = bound_struct_impl_view.impl_block_scope_index;
        
        context.with_new_naming_scope(
            |context| {
                let prev_impl_block_flag = context.name_scoping.is_within_impl_block;
                context.name_scoping.is_within_impl_block = true;
                
                let local_scopes = &mut context.name_scoping.local_scopes;
                let function_scope = context.name_scoping.current_scope;

                for arg in method.arguments.iter() {
                    if local_scopes.is_variable_defined(function_scope, &arg.name.lexeme) {
                        panic!(
                            "Parameter {} is already defined for method {}",
                            arg.name,
                            method.name
                        );
                    }

                    local_scopes.add_variable(
                        function_scope,
                        arg.name.lexeme.clone(),
                        arg.node_id,
                    );
                }

                self.visit_all_statements(&method.body, context);
                context.name_scoping.is_within_impl_block = prev_impl_block_flag;
            }
        );

        context.name_scoping.current_scope = prev_scope;
    }

    fn visit_struct_statement(
        &self,
        struct_statement: &StructStatement,
        context: &mut FirstSemanticsPassContext
    )
    {
        let scope = context.name_scoping.get_current_scope_mut();

        if scope.find_in_scope(&struct_statement.name.lexeme) {
            panic!("Struct {} is already defined in this scope", struct_statement.name.lexeme);
        }

        // TODO: define struct names for further type validation
        scope.structs.insert(struct_statement.name.lexeme.clone(), struct_statement.get_node_id());
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
                .name_scoping
                .local_scopes
                .find_from_current_scope_mut(
                    &struct_name.lexeme
                )
            else {
                panic!(
                    "Name {} was not found in scope for method {}",
                    struct_name.lexeme, struct_name.lexeme
                );
            };

            let Some(struct_declaration_node_id) = struct_scope
                .structs
                .get_mut(&struct_name.lexeme)
            else {
                panic!(
                    "Struct {} was not found in scope for method {}",
                    struct_name.lexeme, struct_name.lexeme
                );
            };

            (struct_scope_idx, *struct_declaration_node_id)
        };
        

        let current_scope = context.name_scoping.get_current_scope_mut();
        current_scope
            .impl_blocks
            .entry(struct_name.lexeme.clone())
            .or_insert(ImplBlock::new(
                impl_statement.implemented_type.lexeme.clone(),
                impl_statement.node_id,
                struct_ast_node_index,
            ));

        context.name_scoping.struct_impl_view.push(StructImplView::new(
            struct_name.lexeme.clone(),
            struct_ast_node_index,
            struct_scope_idx,
            impl_statement.node_id,
            context.name_scoping.current_scope,
        ));

        let prev_impl_block_flag = context.name_scoping.is_within_impl_block;
        context.name_scoping.is_within_impl_block = true;
        // context.with_new_naming_scope(|context|
            self.visit_impl_statement_default(impl_statement, context);
        // );
        context.name_scoping.is_within_impl_block = prev_impl_block_flag
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
