use std::{
    ffi::CString,
    fs,
    mem::{self, MaybeUninit},
    panic,
    path::Path,
    process,
};

use backtrace::Backtrace;

use fluid_mangle::*;
use fluid_parser::{Function, Parser, Prototype, Type};

use llvm::{
    analysis::*,
    core::*,
    execution_engine::*,
    prelude::*,
    transforms::{scalar::*, util::*},
    *,
};

use crate::{
    cstring,
    symbol::{FluidFunctionRef, FluidVariableRef, SymbolTable},
};

const DEBUG: bool = true;

/// Type of codegen to do.
#[derive(Debug, PartialEq)]
pub enum CodeGenType {
    /// Just In Compiled
    JIT {
        /// Run the main function.
        run_main: bool,
    },
    /// Repl
    Repl,
}

/// The internal state when codegen the ast provided by the parser.
pub struct CodeGen {
    /// Reference to the LLVM context.
    pub(crate) context: LLVMContextRef,
    /// Reference to the current module.
    pub(crate) module: LLVMModuleRef,
    /// Reference to the builder.
    pub(crate) builder: LLVMBuilderRef,
    /// Reference to the execution engine.
    pub(crate) execution_engine: LLVMExecutionEngineRef,
    /// Reference to the pass manager.
    pub(crate) pass_manager: LLVMPassManagerRef,
    /// The symbol table.
    pub(crate) symbol_table: SymbolTable,
    /// The codegen type.
    pub(crate) codegen_type: CodeGenType,
}

impl CodeGen {
    /// Create a new codegen context.
    pub fn new<S: Into<String>>(module: S, codegen_type: CodeGenType) -> Self {
        panic::set_hook(Box::new(|info| {
            let backtrace = Backtrace::new();

            eprintln!(
                "error: Internal codegen error.
                info: {}
                backtrace: {:?}

                note: The compiler unexpectedly panicked. this is a bug.
                note: We would appreciate a bug report: https://github.com/fluid-lang/fluid/issues/new",
                info, backtrace
            );
        }));

        let module = cstring!("{}", module.into());

        unsafe {
            llvm::target::LLVM_InitializeNativeTarget();
            llvm::target::LLVM_InitializeNativeAsmPrinter();
            llvm::target::LLVM_InitializeNativeAsmParser();

            LLVMLinkInMCJIT();

            let context = LLVMContextCreate();
            let module = LLVMModuleCreateWithNameInContext(module.as_ptr(), context);
            let builder = LLVMCreateBuilderInContext(context);

            let mut execution_engine = MaybeUninit::uninit();
            let mut err_string = MaybeUninit::uninit();

            if LLVMCreateExecutionEngineForModule(execution_engine.as_mut_ptr(), module, err_string.as_mut_ptr()) == 1 {
                let err_string = err_string.assume_init();

                panic!("{}", CString::from_raw(err_string).to_string_lossy());
            }

            let execution_engine = execution_engine.assume_init();

            let pass_manager = LLVMCreateFunctionPassManagerForModule(module);

            LLVMAddInstructionCombiningPass(pass_manager);
            LLVMAddReassociatePass(pass_manager);
            LLVMAddGVNPass(pass_manager);
            LLVMAddCFGSimplificationPass(pass_manager);
            LLVMAddBasicAliasAnalysisPass(pass_manager);
            LLVMAddPromoteMemoryToRegisterPass(pass_manager);
            LLVMAddInstructionCombiningPass(pass_manager);
            LLVMAddReassociatePass(pass_manager);

            LLVMInitializeFunctionPassManager(pass_manager);

            let symbol_table = SymbolTable::new();

            Self {
                context,
                module,
                builder,
                pass_manager,
                execution_engine,
                codegen_type,
                symbol_table,
            }
        }
    }

    /// Run codegen.
    pub fn run(&mut self, mut parser: Parser) {
        let ast = parser.run();

        unsafe {
            for statement in ast {
                self.gen_statement(statement);
            }

            if let CodeGenType::JIT { run_main } = self.codegen_type {
                if run_main {
                    self.run_main()
                }
            }
        }
    }

    /// Reset the codegen context.
    pub fn reset(&mut self) {}

    /// Emit llvm ir.
    pub fn emit_llvm(&mut self, file: &str) {
        unsafe {
            let file_name = Path::new(file).file_name().unwrap().to_str().unwrap().replace(".fluid", ".ll");
            let ir = CString::from_raw(LLVMPrintModuleToString(self.module));

            fs::write(file_name, ir.to_str().unwrap()).unwrap();
        }
    }

    /// Free all of the resources.
    pub fn free(&mut self) {
        unsafe {
            LLVMContextDispose(self.context);
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMDisposeExecutionEngine(self.execution_engine);
        }
    }

    /// Run the main function.
    unsafe fn run_main(&mut self) -> ! {
        let args = std::env::args().skip(1);
        let argc = args.len() as i64;

        let vec_args = args.map(|string| std::ffi::CString::new(string).unwrap()).collect::<Vec<_>>();
        let argv = vec_args.iter().map(|cstr| cstr.as_ptr() as *const u8).collect::<Vec<_>>();

        let main_function_addr = LLVMGetFunctionAddress(self.execution_engine, cstring!("main").as_ptr());
        let main_function: extern "C" fn(i64, *const *const u8) -> i64 = mem::transmute(main_function_addr);

        process::exit(main_function(argc, argv.as_ptr()) as i32);
    }

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

    /// Generate type.
    pub(crate) unsafe fn gen_type(&mut self, kind: Type) -> LLVMTypeRef {
        match kind {
            Type::Void => LLVMVoidTypeInContext(self.context),
            Type::Number => LLVMInt64TypeInContext(self.context),
            Type::Float => LLVMFloatTypeInContext(self.context),
            Type::String => LLVMPointerType(LLVMInt8TypeInContext(self.context), 0),
            Type::Bool => LLVMInt1TypeInContext(self.context),
        }
    }

    /// Dump the given value.
    #[inline(always)]
    pub(crate) unsafe fn dump_value(&self, value: LLVMValueRef) {
        if DEBUG {
            LLVMDumpValue(value);
        }
    }
}
