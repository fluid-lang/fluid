//! Language builtin items.

use crate::CodeGen;

// TODO: Panic handler
// TODO: Eh personality
// TODO: String, println, etc...

impl CodeGen {
    #[inline]
    pub(crate) unsafe fn init_stdlib(&mut self) {}
}
