//! The `fluid_parser` crate contatins the parser implementation for Fluid.
//! The output by the parser is in an AST (Abstract Syntax Tree)

#![deny(unsafe_code, trivial_numeric_casts, unused_extern_crates, unstable_features)]

mod ast;
mod parser;
mod utils;

pub use ast::*;
pub use parser::*;
