/**
 * Code generation from the BNF AST.
 */

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::Block;
use crate::bnf;

pub fn generate_code(rules: &bnf::RuleSet) -> TokenStream {
    let mut parser_fns = Vec::new();
    for (name, node) in &rules.rules {
        parser_fns.push(generate_code_rule(rules, name, node));
    }
    quote!{
        #(#parser_fns)*
    }
}

fn generate_code_rule(rs: &bnf::RuleSet, name: &str, node: &bnf::Node) -> TokenStream {
    let code = generate_code_node(rs, node);
    let fname = quote::format_ident!("parse_{}", name);
    quote!{
        fn #fname<I>(src: I) where I : ::std::iter::Iterator {
            #code
        }
    }
}

fn generate_code_node(rs: &bnf::RuleSet, node: &bnf::Node) -> TokenStream {
    match node {
        bnf::Node::Transformation{ subnode, action, } =>
            generate_code_transformation(rs, subnode, action),

        bnf::Node::Alternative{ first, second, } =>
            generate_code_alternative(rs, first, second),

        bnf::Node::Sequence{ first, second, } =>
            generate_code_sequence(rs, first, second),

        bnf::Node::Literal(lit) => match lit {
            bnf::LiteralNode::Ident(p) => unimplemented!(),
            bnf::LiteralNode::Lit(l) => unimplemented!(),
        },
    }
}

fn generate_code_transformation(rs: &bnf::RuleSet, node: &bnf::Node, action: &Block) -> TokenStream {
    unimplemented!();
}

fn generate_code_alternative(rs: &bnf::RuleSet, first: &bnf::Node, second: &bnf::Node) -> TokenStream {
    unimplemented!();
}

fn generate_code_sequence(rs: &bnf::RuleSet, first: &bnf::Node, second: &bnf::Node) -> TokenStream {
    unimplemented!();
}
