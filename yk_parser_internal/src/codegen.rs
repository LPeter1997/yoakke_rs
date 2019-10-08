/**
 * Code generation from the BNF AST.
 */

use proc_macro2::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{Ident, Block, Lit, Path};
use crate::bnf;
use crate::parse_result::*;

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

    // TODO: Proper return type
    let ret_ty: Vec<_> = (0..counter).map(|_| quote!{ Box<AST> }).collect();

    quote!{
        fn #fname<I>(src: I, idx: usize) -> ::yk_parser::ParseResult<I, (#(#ret_ty),*)>
            where I : ::std::iter::Iterator + ::std::clone::Clone,
            <I as ::std::iter::Iterator>::Item :
                // TODO: Collect what we compare with!
                  ::std::cmp::PartialEq<char>

                + ::std::fmt::Display
                {

            let curr_rule = #name;
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

    let code = quote!{{
        let res = { #code };
        if let ::yk_parser::ParseResult::Ok(::yk_parser::ParseOk{
            furthest_look, furthest_it, furthest_error, value: (#params) }) = res {
            let value = (#closure)(#params);
            ::yk_parser::ParseResult::Ok(::yk_parser::ParseOk{
                furthest_look, furthest_it, furthest_error, value })
        }
        else {
            ::yk_parser::ParseResult::Err(res.err())
        }
    }};

    (code, 1)
}

fn generate_code_alternative(rs: &bnf::RuleSet, counter: usize,
    first: &bnf::Node, second: &bnf::Node) -> (TokenStream, usize) {

    let (code1, counter1) = generate_code_node(rs, counter, first);
    let (code2, counter2) = generate_code_node(rs, counter, second);

    assert_eq!(counter1, counter2);

    let params = param_list(counter..counter1);

    let code = quote!{{
        let res1 = { #code1 };
        let res2 = { #code2 };
        ::yk_parser::ParseResult::unify_alternatives(res1, res2)
    }};

    (code, counter1)
}

fn generate_code_sequence(rs: &bnf::RuleSet, counter: usize,
    first: &bnf::Node, second: &bnf::Node) -> (TokenStream, usize) {

    let (code1, counter1) = generate_code_node(rs, counter, first);
    let (code2, counter2) = generate_code_node(rs, counter1, second);

    let params1 = param_list(counter..counter1);
    let params2 = param_list(counter1..counter2);

    let code = quote!{{
        let res1 = { #code1 };
        if let ::yk_parser::ParseResult::Ok(ok) = res1 {
            let src = ok.furthest_it.clone();
            let idx = ok.furthest_look;
            let res2 = { #code2 };

            let res_tmp = ::yk_parser::ParseResult::unify_sequence(ok, res2);

            if let ::yk_parser::ParseResult::Ok(::yk_parser::ParseOk{
                furthest_look, furthest_it, furthest_error, value: ((#params1), (#params2)) }) = res_tmp {

                // Flatten
                ::yk_parser::ParseResult::Ok(::yk_parser::ParseOk{
                    furthest_look, furthest_it, furthest_error, value: (#params1, #params2) })
            }
            else {
                ::yk_parser::ParseResult::Err(res_tmp.err())
            }
        }
        else {
            ::yk_parser::ParseResult::Err(res1.err())
        }
    }};

    (code, counter2)
}

fn generate_code_lit(rs: &bnf::RuleSet, counter: usize,
    lit: &Lit) -> (TokenStream, usize) {

    let lit_str = format!("{}", lit.into_token_stream());

    let code = quote!{{
        let mut src2 = src.clone();
        if let Some(v) = src2.next() {
            if v == #lit {
                ::yk_parser::ParseResult::Ok(::yk_parser::ParseOk{
                    furthest_look: idx + 1, furthest_it: src2, furthest_error: None, value: (v) })
            }
            else {
                let got = format!("{}", v);
                ::yk_parser::ParseResult::Err(::yk_parser::ParseErr::single(
                    idx, got, ::std::string::String::from(curr_rule), ::std::string::String::from(#lit_str)))
            }
        }
        else {
            ::yk_parser::ParseResult::Err(::yk_parser::ParseErr::single(
                idx, ::std::string::String::from("end of input"), ::std::string::String::from(curr_rule), ::std::string::String::from(#lit_str)))
        }
    }};

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
                #fname(src.clone(), idx)
            };
            return (code, counter + 1);
        }
    }

    let lit_str = format!("{}", lit.into_token_stream());

    // Some identifier
    let code = quote!{{
        let mut src2 = src.clone();
        if let Some(v) = src2.next() {
            if v == #lit {
                ::yk_parser::ParseResult::Ok(::yk_parser::ParseOk{
                    furthest_look: idx + 1, furthest_it: src2, furthest_error: None, value: (v) })
            }
            else {
                let got = format!("{}", v);
                ::yk_parser::ParseResult::Err(::yk_parser::ParseErr::single(
                    idx, got, ::std::string::String::from(curr_rule), ::std::string::String::from(#lit_str)))
            }
        }
        else {
            ::yk_parser::ParseResult::Err(::yk_parser::ParseErr::single(
                idx, ::std::string::String::from("end of input"), ::std::string::String::from(curr_rule), ::std::string::String::from(#lit_str)))
        }
    }};
    return (code, counter + 1);
}

// Helpers

fn param_list(r: std::ops::Range<usize>) -> TokenStream {
    let params: Vec<_> = r.map(|x| quote::format_ident!("e{}", x)).collect();
    quote!{ #(#params),* }
}
