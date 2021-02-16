use fluid_parser::{Expression, Statement, Type};

use llvm_sys::core::*;

use crate::{cstring, symbol::FluidVariableRef, CodeGen};

impl CodeGen {
    /// Generate the function's body.
    #[inline(always)]
    pub(crate) unsafe fn gen_function_body(&mut self, body: Statement) {
        match body {
            Statement::Block(block) => {
                for statement in block {
                    self.gen_statement(statement);
                }
            }
            _ => unreachable!(),
        }
    }

    /// Generate a statement.
    pub(crate) unsafe fn gen_statement(&mut self, statement: Statement) {
        match statement {
            Statement::Expression(expression) => {
                self.gen_expression(&expression);
            }
            Statement::Return(expression) => self.gen_return_statement(*expression),
            Statement::VarDef(name, kind, value) => self.gen_var_def(name, kind, *value),
            Statement::Block(block) => self.gen_block(block),
            _ => unimplemented!(),
        }
    }

    /// Generate a block statement.
    pub(crate) unsafe fn gen_block(&mut self, block: Vec<Statement>) {
        self.symbol_table.push_scope();

        let mut result = vec![];

        for statement in block {
            result.push(self.gen_statement(statement));
        }

        self.symbol_table.pop_scope();
    }

    /// Generate a return statement.
    pub(crate) unsafe fn gen_return_statement(&mut self, expression: Expression) {
        let expression = self.gen_expression(&expression);

        LLVMBuildRet(self.builder, expression.value);
    }

    /// Generate variable definition.
    pub(crate) unsafe fn gen_var_def(&mut self, name: String, kind: Type, value: Expression) {
        let llvm_type = self.gen_type(kind);
        let var_value = self.gen_expression(&value);

        let variable_alloca = LLVMBuildAlloca(self.builder, llvm_type, cstring!("{}", name).as_ptr());
        LLVMBuildStore(self.builder, var_value.value, variable_alloca);

        let variable_ref = FluidVariableRef::new(true, kind, variable_alloca);

        self.symbol_table.insert_variable(name, variable_ref);
    }
}
