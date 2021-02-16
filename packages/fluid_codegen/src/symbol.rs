use std::collections::HashMap;

use fluid_parser::Type;
use llvm_sys::prelude::LLVMValueRef;

/// The scope's unique id.
type ScopeId = usize;

/// The symbol table.
#[derive(Debug)]
pub(crate) struct SymbolTable {
    /// All of the scopes.
    scopes: Vec<Scope>,
    /// The current scope's id.
    current: ScopeId,
}

impl SymbolTable {
    /// Create a new SymbolTable
    pub(crate) fn new() -> Self {
        let global = Scope::new(None);
        let scopes = vec![global];
        let current = 0;

        Self { scopes, current }
    }

    /// Push a new scope in the symbol table.
    pub(crate) fn push_scope(&mut self) {
        self.scopes.push(Scope::new(Some(self.current)));
        self.current += 1;
    }

    /// Pop the current scope.
    pub(crate) fn pop_scope(&mut self) {
        self.current -= 1;
    }

    /// Get the current scope.
    pub(crate) fn current_scope(&mut self) -> &mut Scope {
        &mut self.scopes[self.current]
    }

    /// Get the parent of the current scope.
    pub(crate) fn current_scope_parent(&mut self) -> &mut Scope {
        let current = self.current_scope();
        let parent_id = current.parent.unwrap();

        &mut self.scopes[parent_id]
    }

    /// Insert a function in the current scope.
    pub(crate) fn insert_function(&mut self, function_name: String, function_ref: FluidFunctionRef) {
        let current = self.current_scope();

        current.insert_function(function_name, function_ref);
    }

    /// Insert a variable in the current scope.
    pub(crate) fn insert_variable(&mut self, variable_name: String, variable_ref: FluidVariableRef) {
        let current = self.current_scope();

        current.insert_variable(variable_name, variable_ref);
    }

    /// Get a variable in the scope.
    pub(crate) fn get_variable(&mut self, variable_name: &str) -> Option<&FluidVariableRef> {
        let current = self.current_scope();

        current.get_variable(variable_name)
    }

    /// Get a function in the scope.
    pub(crate) fn get_function(&mut self, function_name: &str) -> Option<&FluidFunctionRef> {
        let current = self.current_scope();

        current.get_function(function_name)
    }
}

/// A scope
#[derive(Debug)]
pub(crate) struct Scope {
    /// ScopeId of the parent scope.
    /// The ScopeId of the parent scope will be None if its the global scope.
    pub(crate) parent: Box<Option<ScopeId>>,

    /// List of all of the functions in the scope.
    functions: HashMap<String, FluidFunctionRef>,
    /// List of all of the variables in the scope.
    variables: HashMap<String, FluidVariableRef>,
}

impl Scope {
    /// Create a new scope.
    pub(crate) fn new(parent: Option<ScopeId>) -> Self {
        let parent = Box::new(parent);

        let functions = HashMap::new();
        let variables = HashMap::new();

        Self { parent, functions, variables }
    }

    /// Insert a new function in the scope.
    #[inline(always)]
    pub(crate) fn insert_function(&mut self, function_name: String, function_ref: FluidFunctionRef) {
        self.functions.insert(function_name, function_ref);
    }

    /// Insert a new variable in the scope.
    #[inline(always)]
    pub(crate) fn insert_variable(&mut self, variable_name: String, variable_ref: FluidVariableRef) {
        self.variables.insert(variable_name, variable_ref);
    }

    /// Get a variable in the scope.
    #[inline(always)]
    pub(crate) fn get_variable(&self, variable_name: &str) -> Option<&FluidVariableRef> {
        self.variables.get(variable_name)
    }

    /// Get a variable in the scope.
    #[inline(always)]
    pub(crate) fn get_function(&self, function_name: &str) -> Option<&FluidFunctionRef> {
        self.functions.get(function_name)
    }
}

/// Fluid variable reference.
#[derive(Debug)]
pub(crate) struct FluidVariableRef {
    /// Is the variable initialized.
    pub(crate) initialized: bool,
    /// The type of the variable.
    pub(crate) kind: Type,
    /// The alloca of the variable.
    pub(crate) alloca: LLVMValueRef,
}

impl FluidVariableRef {
    /// Create a new variable reference.
    pub(crate) fn new(initialized: bool, kind: Type, alloca: LLVMValueRef) -> Self {
        Self { initialized, kind, alloca }
    }
}

/// Fluid function reference.
#[derive(Debug)]
pub(crate) struct FluidFunctionRef {
    /// Args of the function.
    pub(crate) args: Vec<Type>,
    /// Return type of the function.
    pub(crate) return_type: Type,
    /// Value of the generated function.
    pub(crate) value: LLVMValueRef,
}

impl FluidFunctionRef {
    /// Create a new function reference.
    pub(crate) fn new(args: Vec<Type>, return_type: Type, value: LLVMValueRef) -> Self {
        Self { args, return_type, value }
    }
}
