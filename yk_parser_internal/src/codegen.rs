/**
 * Code generation from the BNF AST.
 */

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use crate::bnf;

pub fn generate_code(rules: &bnf::RuleSet) -> TokenStream {
    let mut parser_fns = Vec::new();
    for (name, node) in &rules.rules {
        parser_fns.push(generate_code_rule(name, node));
    }
    quote!{
        #(#parser_fns)*
    }
}

fn generate_code_rule(name: &str, node: &bnf::Node) -> TokenStream {
    let code = generate_code_node(node);
    let fname = quote::format_ident!("parse_{}", name);
    quote!{
        fn #fname<I>(src: I) where I : Iterator {
            #code
        }
    }
}

fn generate_code_node(node: &bnf::Node) -> TokenStream {
    //unimplemented!();
    quote!{}
}
