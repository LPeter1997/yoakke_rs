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

        memo_members.push(quote!{ #memo_id: HashMap<usize, #memo_ty> });
        memo_ctor.push(quote!{ #memo_id: HashMap::new() });
    }

    quote!{
        mod parser {
            use ::yk_parser::{irec, drec, ParseResult, ParseOk, ParseErr};
            use ::std::string::String;
            use ::std::option::Option;
            use ::std::collections::HashMap;
            use ::std::iter::Iterator;
            use ::std::clone::Clone;
            use ::std::cmp::PartialEq;
            // TODO: Is this temporary
            use ::std::fmt::Display;

            pub struct MemoContext<I> {
                #(#memo_members),*
            }

            impl <I> MemoContext<I> {
                pub fn new() -> Self {
                    Self{
                        #(#memo_ctor),*
                    }
                }
            }

            #(#parser_fns)*
        }
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
            ParseResult<I, #ret_ty>
        }},

        bnf::LeftRecursion::Direct => {quote!{
            drec::DirectRec<I, #ret_ty>
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
                    memo.#memo_id.insert(idx, drec::DirectRec::Base(ParseResult::Err(ParseErr::new()), true));
                    // Now invoke the parser
                    // If it's recursive, the entry must have changed
                    let tmp_res = { #code };
                    // Refresh the entry, check
                    match memo.#memo_id.get(&idx).unwrap() {
                        drec::DirectRec::Recurse(_) => {
                            // We are in recursion!
                            memo.#memo_id.insert(idx, drec::DirectRec::Recurse(tmp_res));
                            let old = memo.#memo_id.get(&idx).unwrap().parse_result();
                            #grow_fname(memo, src.clone(), idx, old.clone())
                        },

                        drec::DirectRec::Base(_, _) => {
                            // No change, write back result
                            // Overwrite the base-type to contain the result
                            memo.#memo_id.insert(idx, drec::DirectRec::Base(tmp_res, false));
                            let inserted = memo.#memo_id.get(&idx).unwrap();
                            inserted.parse_result().clone()
                        }
                    }
                },

                Some(drec::DirectRec::Base(res, true)) => {
                    // Recursion signal, write back a dummy error to start!
                    // TODO: Instead of cloning we could just remove it from here!
                    memo.#memo_id.insert(idx, drec::DirectRec::Recurse(ParseResult::Err(ParseErr::new())));
                    let inserted = memo.#memo_id.get(&idx).unwrap();
                    inserted.parse_result().clone()
                },

                Some(drec::DirectRec::Base(res, false)) => {
                    res.clone()
                },

                Some(drec::DirectRec::Recurse(res)) => {
                    res.clone()
                }
            }
        }},

        bnf::LeftRecursion::Indirect => { unimplemented!(); }
    };

    let fn_where_clause = quote!{
        I : Iterator + Clone,
        <I as Iterator>::Item :
            // TODO: Collect what we compare with!
              PartialEq<char>
            + Display
    };

    // Generate extra functions
    let grow_code = match rec {
        bnf::LeftRecursion::None => quote!{},

        bnf::LeftRecursion::Direct => {quote!{
            fn #grow_fname<I>(memo: &mut MemoContext<I>, src: I, idx: usize,
                old: ParseResult<I, #ret_ty>) -> ParseResult<I, #ret_ty>
                where #fn_where_clause {

                let curr_rule = #name;

                if old.is_err() {
                    return old.clone();
                }

                let old_ok = old.ok().unwrap();
                let tmp_res = { #code };

                // TODO: Oof, unnecessary cloning
                if tmp_res.is_ok() {
                    let tmp_ok = tmp_res.ok().unwrap();
                    if old_ok.furthest_look() < tmp_ok.furthest_look() {
                        // Successfully grew the seed
                        memo.#memo_id.insert(idx, drec::DirectRec::Recurse(ParseResult::Ok(tmp_ok)));
                        let new_old = memo.#memo_id.get(&idx).unwrap().parse_result();
                        return #grow_fname(memo, src, idx, new_old.clone());
                    }
                    else {
                        // We need to overwrite max-furthest in the memo-table!
                        // That's why we don't simply return old_res
                        let updated = ParseResult::unify_alternatives(
                            ParseResult::Ok(tmp_ok), ParseResult::Ok(old_ok));
                        memo.#memo_id.insert(idx, drec::DirectRec::Recurse(updated));
                        let inserted = memo.#memo_id.get(&idx).unwrap().parse_result();
                        return inserted.clone();
                    }
                }
                else {
                    // We need to overwrite max-furthest in the memo-table!
                    // That's why we don't simply return old_res
                    let updated = ParseResult::unify_alternatives(
                        tmp_res, ParseResult::Ok(old_ok));
                    memo.#memo_id.insert(idx, drec::DirectRec::Recurse(updated));
                    let inserted = memo.#memo_id.get(&idx).unwrap().parse_result();
                    return inserted.clone();
                }
            }
        }},

        bnf::LeftRecursion::Indirect => { unimplemented!(); },
    };

    let parser_fn = quote!{
        #grow_code

        pub fn #fname<I>(memo: &mut MemoContext<I>, src: I, idx: usize) ->
            ParseResult<I, #ret_ty>
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
        if let ParseResult::Ok(ok) = res {
            ok.map(|(#params)| (#closure)(#params)).into()
        }
        else {
            res.err().unwrap().into()
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
        ParseResult::unify_alternatives(res1, res2)
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
        if let ParseResult::Ok(ok) = res1 {
            // Overwrite positional data for the next part's invocation
            let src = ok.furthest_it.clone();
            let idx = ok.matched;
            // Invoke RHS
            let res2 = { #code2 };

            let res_tmp = ParseResult::unify_sequence(ok, res2);

            if let ParseResult::Ok(ok) = res_tmp {
                // Flatten
                ok.map(|((#params1), (#params2))| (#params1, #params2)).into()
            }
            else {
                res_tmp.err().unwrap().into()
            }
        }
        else {
            res1.err().unwrap().into()
        }
    }};

    (code, counter2)
}

fn generate_code_lit(rs: &bnf::RuleSet, counter: usize,
    lit: &Lit) -> (TokenStream, usize) {

    generate_code_atom(counter, quote!{ #lit })
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

    // Some identifier
    return generate_code_atom(counter, quote!{ #lit });
}

fn generate_code_atom(counter: usize, tok: TokenStream) -> (TokenStream, usize) {
    let lit_str = format!("{}", tok);
    let code = quote!{{
        let mut src2 = src.clone();
        if let Some(v) = src2.next() {
            if v == #tok {
                ParseOk{ matched: idx + 1, furthest_it: src2, furthest_error: None, value: (v) }.into()
            }
            else {
                let got = format!("{}", v);
                ParseErr::single(idx, got, curr_rule, #lit_str.into()).into()
            }
        }
        else {
            ParseErr::single(idx, "end of input".into(), curr_rule, #lit_str.into()).into()
        }
    }};
    (code, counter + 1)
}

// Helpers

fn param_list(r: std::ops::Range<usize>) -> TokenStream {
    let params: Vec<_> = r.map(|x| quote::format_ident!("e{}", x)).collect();
    quote!{ #(#params),* }
}
