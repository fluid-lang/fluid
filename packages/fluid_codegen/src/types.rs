use fluid_parser::Type;

use llvm::{core::*, prelude::*};

use crate::CodeGen;

impl CodeGen {
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
}
