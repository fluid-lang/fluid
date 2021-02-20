use fluid_parser::Type;
use llvm::prelude::LLVMValueRef;

/// Convert a string to CString
#[macro_export]
macro_rules! cstring {
    ($($arg:tt)*) => {{
        use std::ffi::CString;
        CString::new(format!($($arg)*)).unwrap()
    }};
}

/// Reference to a fluid value.
#[derive(Debug)]
pub(crate) struct FluidValueRef {
    /// The fluid type of the value.
    pub(crate) kind: Type,
    /// The llvm value.
    pub(crate) value: LLVMValueRef,
}

impl FluidValueRef {
    /// Create a new value reference.
    pub(crate) fn new(kind: Type, value: LLVMValueRef) -> Self {
        Self { kind, value }
    }
}
