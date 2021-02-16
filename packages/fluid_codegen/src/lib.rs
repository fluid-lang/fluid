//! This crate converts the generated AST by `fluid_parser` to LLVM ir.

#![deny(missing_docs, trivial_numeric_casts, unused_extern_crates, unstable_features)]

mod codegen;
mod expression;
mod statement;
mod symbol;
mod utils;

extern crate llvm_sys as llvm;

pub use codegen::*;
