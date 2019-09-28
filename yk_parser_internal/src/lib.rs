
extern crate syn;
extern crate quote;
extern crate proc_macro2;

pub mod bnf;
mod codegen;

pub use codegen::generate_code;
