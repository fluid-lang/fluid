use crate::symbol::*;
use crate::*;

use fluid_mangle::mangle_function_name;
use fluid_parser::{Function, Prototype, Type};
use llvm::{analysis::*, core::*, prelude::*, *};

use crate::CodeGen;

impl CodeGen {
    /// Generate the function prototype.
    pub(crate) unsafe fn gen_prototype(&mut self, prototype: &Prototype) -> LLVMValueRef {
        let return_type = self.gen_type(prototype.return_type);
        let mut argument_types = prototype.args.iter().map(|arg| self.gen_type(arg.typee)).collect::<Vec<_>>();

        let function_type = LLVMFunctionType(return_type, argument_types.as_mut_ptr(), prototype.args.len() as u32, 0);
        let function_value = LLVMAddFunction(self.module, cstring!("{}", prototype.name.as_str()).as_ptr(), function_type);

        LLVMSetLinkage(function_value, LLVMLinkage::LLVMExternalLinkage);

        for i in 0..prototype.args.len() {
            let arg = &prototype.args[i];

            let param = LLVMGetParam(function_value, i as u32);
            LLVMSetValueName2(param, cstring!("{}", arg.name).as_ptr(), arg.name.len());
        }

        if LLVMRunFunctionPassManager(self.pass_manager, function_value) == 1 {
            panic!("Running FunctionPassManager failed.")
        }

        function_value
    }

    /// Generate the function definition.
    pub(crate) unsafe fn gen_function_def(&mut self, mut function: Function) {
        function.prototype.name = mangle_function_name(function.prototype.name, function.prototype.args.iter().map(|arg| arg.typee).collect::<Vec<_>>());

        let function_name = function.prototype.name.clone();
        let function_value = self.gen_prototype(&function.prototype);

        self.symbol_table.push_scope();

        let entry = LLVMAppendBasicBlockInContext(self.context, function_value, cstring!("entry").as_ptr());
        LLVMPositionBuilderAtEnd(self.builder, entry);

        for i in 0..function.prototype.args.len() {
            let arg = &function.prototype.args[i];

            let param = LLVMGetParam(function_value, i as u32);
            let kind = self.gen_type(arg.typee);

            let variable_alloca = LLVMBuildAlloca(self.builder, kind, cstring!("{}", arg.name).as_ptr());
            LLVMBuildStore(self.builder, param, variable_alloca);

            let variable_ref = FluidVariableRef::new(true, arg.typee, variable_alloca);

            self.symbol_table.insert_variable(arg.name.clone(), variable_ref);
        }

        let function_ref = FluidFunctionRef::new(function.prototype.args.iter().map(|arg| arg.typee).collect::<Vec<_>>(), function.prototype.return_type, function_value);

        self.symbol_table.insert_function(function_name, function_ref);
        self.gen_function_body(function.body);

        self.symbol_table.pop_scope();

        if function.prototype.return_type == Type::Void {
            LLVMBuildRetVoid(self.builder);
        }

        // Dump the generated ir.
        self.dump_value(function_value);

        if LLVMVerifyFunction(function_value, LLVMVerifierFailureAction::LLVMReturnStatusAction) == 1 {
            LLVMDeleteFunction(function_value);
            panic!("Fluid generated invalid function ir.")
        }
    }

    /// Generate an external definition.
    pub(crate) unsafe fn gen_extern_def(&mut self, prototype: Prototype) {
        let external_function = self.gen_prototype(&prototype);
        self.dump_value(external_function);
    }
}
