/**
 * Code generation from the BNF AST.
 */

use proc_macro2::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{Ident, Block, Lit, Path};
use crate::bnf;
use crate::parse_result::*;

struct GeneratedRule {
    parser_fn: TokenStream,
    memo_id: Ident,
    memo_ty: TokenStream,
}

// TODO: Eliminate cloning where possible
// TODO: Cleanup

pub fn generate_code(rules: &bnf::RuleSet) -> TokenStream {
    let mut parser_fns = Vec::new();

    let mut memo_members = Vec::new();
    let mut memo_ctor = Vec::new();

    for (name, node) in &rules.rules {
        let GeneratedRule{ parser_fn, memo_id, memo_ty } = generate_code_rule(rules, name, node);

        parser_fns.push(parser_fn);

        memo_members.push(quote!{ #memo_id: ::std::collections::HashMap<usize, #memo_ty> });
        memo_ctor.push(quote!{ #memo_id: ::std::collections::HashMap::new() });
    }

    quote!{
        struct MemoContext<I> {
            #(#memo_members),*
        }

        impl <I> MemoContext<I> {
            fn new() -> Self {
                Self{
                    #(#memo_ctor),*
                }
            }
        }

        #(#parser_fns)*
    }
}

fn generate_code_rule(rs: &bnf::RuleSet,
    name: &str, node: &bnf::Node) -> GeneratedRule {

    let (code, counter) = generate_code_node(rs, 0, node);

    let fname = quote::format_ident!("parse_{}", name);
    let grow_fname = quote::format_ident!("grow_{}", name);
    let recall_fname = quote::format_ident!("recall_{}", name);
    let setup_lr_fname = quote::format_ident!("setup_lr_{}", name);
    let lr_answer_fname = quote::format_ident!("lr_answer_{}", name);
    let memo_id = quote::format_ident!("memo_{}", name);

    // TODO: Proper return type
    let ret_tys: Vec<_> = (0..counter).map(|_| quote!{ i32 }).collect();
    let ret_ty = quote!{ (#(#ret_tys),*) };

    let rec = rs.left_recursion(name);
    let memo_ty = match rec {
        bnf::LeftRecursion::None => {quote!{
            ::yk_parser::ParseResult<I, #ret_ty>
        }},

        bnf::LeftRecursion::Direct => {quote!{
            ::yk_parser::DirectRec<I, #ret_ty>
        }},

        bnf::LeftRecursion::Indirect => {
            unimplemented!();
        }
    };

    let memo_code = match rec {
        bnf::LeftRecursion::None => {quote!{
            // TODO: Oof... We are cloning the result!
            if let Some(res) = memo.#memo_id.get(&idx) {
                res.clone()
            }
            else {
                let res = { #code };
                memo.#memo_id.insert(idx, res);
                let inserted = memo.#memo_id.get(&idx).unwrap();
                inserted.clone()
            }
        }},

        bnf::LeftRecursion::Direct => {quote!{
            // TODO: Oof... We are cloning the result!
            match memo.#memo_id.get(&idx) {
                None => {
                    // Nothing is in the cache, write a dummy error
                    memo.#memo_id.insert(idx,
                        ::yk_parser::DirectRec::Base(::yk_parser::ParseResult::Err(::yk_parser::ParseErr::new()), true));
                    // Now invoke the parser
                    // If it's recursive, the entry must have changed
                    let tmp_res = { #code };
                    // Refresh the entry, check
                    match memo.#memo_id.get(&idx).unwrap() {
                        ::yk_parser::DirectRec::Recurse(_) => {
                            // We are in recursion!
                            memo.#memo_id.insert(idx, ::yk_parser::DirectRec::Recurse(tmp_res));
                            let old = memo.#memo_id.get(&idx).unwrap().parse_result();
                            #grow_fname(memo, src.clone(), idx, old.clone())
                        },

                        ::yk_parser::DirectRec::Base(_, _) => {
                            // No change, write back result
                            // Overwrite the base-type to contain the result
                            memo.#memo_id.insert(idx, ::yk_parser::DirectRec::Base(tmp_res, false));
                            let inserted = memo.#memo_id.get(&idx).unwrap();
                            inserted.parse_result().clone()
                        }
                    }
                },

                Some(::yk_parser::DirectRec::Base(res, true)) => {
                    // Recursion signal, write back a dummy error to start!
                    // TODO: Instead of cloning we could just remove it from here!
                    memo.#memo_id.insert(idx, ::yk_parser::DirectRec::Recurse(::yk_parser::ParseResult::Err(::yk_parser::ParseErr::new())));
                    let inserted = memo.#memo_id.get(&idx).unwrap();
                    inserted.parse_result().clone()
                },

                Some(::yk_parser::DirectRec::Base(res, false)) => {
                    res.clone()
                },

                Some(::yk_parser::DirectRec::Recurse(res)) => {
                    res.clone()
                }
            }
        }},

        bnf::LeftRecursion::Indirect => { unimplemented!(); }
    };

    let fn_where_clause = quote!{
        I : ::std::iter::Iterator + ::std::clone::Clone,
        <I as ::std::iter::Iterator>::Item :
            // TODO: Collect what we compare with!
              ::std::cmp::PartialEq<char>

            + ::std::fmt::Display
    };

    // Generate extra functions
    let grow_code = match rec {
        bnf::LeftRecursion::None => quote!{},

        bnf::LeftRecursion::Direct => {quote!{
            fn #grow_fname<I>(memo: &mut MemoContext<I>, src: I, idx: usize,
                old: ::yk_parser::ParseResult<I, #ret_ty>) -> ::yk_parser::ParseResult<I, #ret_ty>
                where #fn_where_clause {

                let curr_rule = #name;

                if old.is_err() {
                    return old.clone();
                }

                let old_ok = old.ok();
                let tmp_res = { #code };

                // TODO: Oof, unnecessary cloning
                if tmp_res.is_ok() {
                    let tmp_ok = tmp_res.ok();
                    if old_ok.furthest_look() < tmp_ok.furthest_look() {
                        // Successfully grew the seed
                        memo.#memo_id.insert(idx, ::yk_parser::DirectRec::Recurse(::yk_parser::ParseResult::Ok(tmp_ok)));
                        let new_old = memo.#memo_id.get(&idx).unwrap().parse_result();
                        return #grow_fname(memo, src, idx, new_old.clone());
                    }
                    else {
                        // We need to overwrite max-furthest in the memo-table!
                        // That's why we don't simply return old_res
                        let updated = ::yk_parser::ParseResult::unify_alternatives(
                            ::yk_parser::ParseResult::Ok(tmp_ok), ::yk_parser::ParseResult::Ok(old_ok));
                        memo.#memo_id.insert(idx, ::yk_parser::DirectRec::Recurse(updated));
                        let inserted = memo.#memo_id.get(&idx).unwrap().parse_result();
                        return inserted.clone();
                    }
                }
                else {
                    // We need to overwrite max-furthest in the memo-table!
                    // That's why we don't simply return old_res
                    let updated = ::yk_parser::ParseResult::unify_alternatives(
                        tmp_res, ::yk_parser::ParseResult::Ok(old_ok));
                    memo.#memo_id.insert(idx, ::yk_parser::DirectRec::Recurse(updated));
                    let inserted = memo.#memo_id.get(&idx).unwrap().parse_result();
                    return inserted.clone();
                }
            }
        }},

        bnf::LeftRecursion::Indirect => { unimplemented!(); },
    };

    let parser_fn = quote!{
        #grow_code

        fn #fname<I>(memo: &mut MemoContext<I>, src: I, idx: usize) ->
            ::yk_parser::ParseResult<I, #ret_ty>
            where #fn_where_clause {

            let curr_rule = #name;
            #memo_code
        }
    };

    GeneratedRule{ parser_fn, memo_id, memo_ty }
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
                #fname(memo, src.clone(), idx)
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
