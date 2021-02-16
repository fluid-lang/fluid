//! The `fluid_lexer` crate contains the main `Token` and `Lexer` interfaces.

#![deny(unsafe_code, trivial_numeric_casts, unused_extern_crates, unstable_features, missing_docs)]

mod lexer;
mod token;
mod utils;

#[cfg(test)]
mod tests;

pub use lexer::*;
pub use token::*;
