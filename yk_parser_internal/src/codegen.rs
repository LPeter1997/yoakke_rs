/**
 * Code generation from the BNF AST.
 */

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{Ident, Block, Lit, Path};
use crate::bnf;

pub fn generate_code(rules: &bnf::RuleSet) -> TokenStream {
    let mut parser_fns = Vec::new();
    for (name, node) in &rules.rules {
        parser_fns.push(generate_code_rule(rules, name, node));
    }
    quote!{
        trait Promoter {}
        impl <T> Promoter for T {}

        #(#parser_fns)*
    }
}

fn generate_code_rule(rs: &bnf::RuleSet,
    name: &str, node: &bnf::Node) -> TokenStream {

    let (code, counter) = generate_code_node(rs, 0, node);
    let fname = quote::format_ident!("parse_{}", name);

    let ret_ty: Vec<_> = (0..counter).map(|_| quote!{ impl Promoter }).collect();

    quote!{
        fn #fname<I>(src: I) -> ::std::result::Result<(I, (#(#ret_ty),*)), ()>
            where I : ::std::iter::Iterator + ::std::clone::Clone,
            <I as std::iter::Iterator>::Item :
                // TODO: Collect what!
                  ::std::cmp::PartialEq<char>
                {

            #code
        }
    }
}

fn generate_code_node(rs: &bnf::RuleSet, counter: usize,
    node: &bnf::Node) -> (TokenStream, usize) {

    match node {
        bnf::Node::Transformation{ subnode, action, } =>
            generate_code_transformation(rs, counter, subnode, action),

        bnf::Node::Alternative{ first, second, } =>
            generate_code_alternative(rs, counter, first, second),

        bnf::Node::Sequence{ first, second, } =>
            generate_code_sequence(rs, counter, first, second),

        bnf::Node::Literal(lit) => match lit {
            bnf::LiteralNode::Ident(p) => generate_code_ident(rs, counter, p),
            bnf::LiteralNode::Lit(l) => generate_code_lit(rs, counter, l),
        },
    }
}

fn generate_code_transformation(rs: &bnf::RuleSet, counter: usize,
    node: &bnf::Node, action: &Block) -> (TokenStream, usize) {

    assert_eq!(counter, 0);

    let (code, counter) = generate_code_node(rs, counter, node);

    let params = param_list(0..counter);
    let closure = quote!{ |#params| #action };

    let code = quote!{
        if let ::std::result::Result::Ok((src, (#params))) = { #code } {
            let val = (#closure)(#params);
            ::std::result::Result::Ok((src, (val)))
        }
        else {
            ::std::result::Result::Err(())
        }
    };

    (code, 1)
}

fn generate_code_alternative(rs: &bnf::RuleSet, counter: usize,
    first: &bnf::Node, second: &bnf::Node) -> (TokenStream, usize) {

    let (code1, counter1) = generate_code_node(rs, counter, first);
    let (code2, counter2) = generate_code_node(rs, counter, second);

    assert_eq!(counter1, counter2);

    let params = param_list(counter..counter1);

    let code = quote!{
        if let ::std::result::Result::Ok((src, (#params))) = { #code1 } {
            ::std::result::Result::Ok((src, (#params)))
        }
        else if let ::std::result::Result::Ok((src, (#params))) = { #code2 } {
            ::std::result::Result::Ok((src, (#params)))
        }
        else {
            ::std::result::Result::Err(())
        }
    };

    (code, counter1)
}

fn generate_code_sequence(rs: &bnf::RuleSet, counter: usize,
    first: &bnf::Node, second: &bnf::Node) -> (TokenStream, usize) {

    let (code1, counter1) = generate_code_node(rs, counter, first);
    let (code2, counter2) = generate_code_node(rs, counter1, second);

    let params1 = param_list(counter..counter1);
    let params2 = param_list(counter1..counter2);

    let code = quote!{
        if let ::std::result::Result::Ok((src, (#params1))) = { #code1 } {
            if let ::std::result::Result::Ok((src, (#params2))) = { #code2 } {
                ::std::result::Result::Ok((src, (#params1, #params2)))
            }
            else {
                ::std::result::Result::Err(())
            }
        }
        else {
            ::std::result::Result::Err(())
        }
    };

    (code, counter2)
}

fn generate_code_lit(rs: &bnf::RuleSet, counter: usize,
    lit: &Lit) -> (TokenStream, usize) {

    let code = quote!{
        let mut src2 = src.clone();
        if let Some(v) = src2.next() {
            if v == #lit {
                ::std::result::Result::Ok((src2, (v)))
            }
            else {
                ::std::result::Result::Err(())
            }
        }
        else {
            ::std::result::Result::Err(())
        }
    };

    (code, counter + 1)
}

fn generate_code_ident(rs: &bnf::RuleSet, counter: usize,
    lit: &Path) -> (TokenStream, usize) {

    // Rule identifier
    if lit.leading_colon.is_none() && lit.segments.len() == 1 {
        let id = lit.segments[0].ident.to_string();
        if rs.rules.contains_key(&id) {
            let fname = quote::format_ident!("parse_{}", id);
            let code = quote!{
                #fname(src)
            };
            return (code, counter + 1);
        }
    }

    // Some identifier
    let code = quote!{
        let mut src2 = src.clone();
        if let Some(v) = src2.next() {
            if v == #lit {
                ::std::result::Result::Ok((src2, (v)))
            }
            else {
                ::std::result::Result::Err(())
            }
        }
        else {
            ::std::result::Result::Err(())
        }
    };
    return (code, counter + 1);
}

// Helpers

fn param_list(r: std::ops::Range<usize>) -> TokenStream {
    let params: Vec<_> = r.map(|x| quote::format_ident!("e{}", x)).collect();
    quote!{ #(#params),* }
}
