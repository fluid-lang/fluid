use std::{
    ffi::CString,
    fs,
    mem::{self, MaybeUninit},
    panic,
    path::Path,
    process, ptr,
};

use backtrace::Backtrace;

use fluid_parser::{Expression, Parser, Statement};

use llvm::{
    core::*,
    execution_engine::*,
    prelude::*,
    target_machine::*,
    transforms::{scalar::*, util::*},
    *,
};

use crate::{cstring, symbol::SymbolTable};

#[cfg(debug_assertions)]
const DEBUG: bool = true;

#[cfg(not(debug_assertions))]
const DEBUG: bool = false;

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
    /// The target machine.
    pub(crate) target_machine: LLVMTargetMachineRef,
}

impl CodeGen {
    /// Create a new codegen context.
    pub fn new<S: Into<String>>(module: S, codegen_type: CodeGenType) -> Self {
        // Set the panic hook.
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
            // Initialize LLVM.
            llvm::target::LLVM_InitializeAllTargetInfos();
            llvm::target::LLVM_InitializeAllTargets();
            llvm::target::LLVM_InitializeAllTargetMCs();
            llvm::target::LLVM_InitializeAllAsmParsers();
            llvm::target::LLVM_InitializeAllAsmPrinters();

            // Get the default target triple of the machine.
            let target_triple = target_machine::LLVMGetDefaultTargetTriple();

            let mut target = ptr::null_mut();
            let mut error_str = MaybeUninit::uninit();

            if target_machine::LLVMGetTargetFromTriple(target_triple, &mut target, error_str.as_mut_ptr()) == 1 {
                let error_str = error_str.assume_init();

                println!("{}", CString::from_raw(error_str).to_string_lossy())
            }

            let opt_level = LLVMCodeGenOptLevel::LLVMCodeGenLevelNone;
            let reloc_mode = LLVMRelocMode::LLVMRelocDefault;
            let code_model = LLVMCodeModel::LLVMCodeModelDefault;

            let cpu = cstring!("native").as_ptr();
            let features = cstring!("").as_ptr();

            let target_machine = LLVMCreateTargetMachine(target, target_triple, cpu, features, opt_level, reloc_mode, code_model);

            LLVMLinkInMCJIT();

            let context = LLVMContextCreate();
            let module = LLVMModuleCreateWithNameInContext(module.as_ptr(), context);
            let builder = LLVMCreateBuilderInContext(context);

            LLVMSetTarget(module, target_triple);

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
                target_machine,
            }
        }
    }

    /// Run codegen.
    pub fn run(&mut self, mut parser: Parser) {
        let ast = parser.run();

        unsafe {
            self.init_stdlib();

            match self.codegen_type {
                CodeGenType::JIT { run_main } => {
                    for statement in ast {
                        self.gen_statement(statement);
                    }

                    if run_main {
                        self.run_main()
                    }
                }
                CodeGenType::Repl => {
                    for statement in ast {
                        if let Statement::Expression(expression) = statement {
                            self.run_top_level_expression(&expression);
                        } else {
                            self.gen_statement(statement);
                        }
                    }
                }
            }
        }
    }

    /// Reset the codegen context.
    pub fn reset(&mut self) {}

    /// Emit LLVM IR.
    pub fn emit_llvm(&mut self, file: &str) {
        unsafe {
            let file_name = Path::new(file).file_name().unwrap().to_str().unwrap().replace(".fluid", ".ll");
            let ir = CString::from_raw(LLVMPrintModuleToString(self.module));

            fs::write(file_name, ir.to_str().unwrap()).unwrap();
        }
    }

    /// Emit an object file.
    pub fn emit_object(&mut self, path: &Path) {
        let mut error_str = MaybeUninit::uninit();
        let file_name = cstring!("{}", path.to_string_lossy()).into_raw();

        unsafe {
            LLVMTargetMachineEmitToFile(self.target_machine, self.module, file_name, LLVMCodeGenFileType::LLVMObjectFile, error_str.as_mut_ptr());
        }
    }

    /// Free all of the resources.
    pub fn free(&mut self) {
        unsafe {
            LLVMContextDispose(self.context);
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMDisposeExecutionEngine(self.execution_engine);

            LLVMShutdown();
        }
    }

    unsafe fn run_top_level_expression(&mut self, _expression: &Expression) {}

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

    /// Dump the given value.
    #[inline]
    pub(crate) unsafe fn dump_value(&self, value: LLVMValueRef) {
        if DEBUG {
            LLVMDumpValue(value);
        }
    }
}
