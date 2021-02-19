//! This crate is responsible for mangling the function names and the class names.
//! Name mangling is a technique used to solve various problems caused by the need to resolve unique names for programming entities.
//!
//! For more information about name mangling: https://en.wikipedia.org/wiki/Name_mangling

use fluid_parser::Type;

/// Mangle a function name.
pub fn mangle_function_name(name: String, _params: Vec<Type>) -> String {
    name
}
