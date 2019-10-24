/**
 * Code generation from the BNF AST.
 */

use std::time::SystemTime;
use proc_macro2::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{Ident, Block, Lit, Path, Type};
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

    // Identifier for type
    let memo_ctx = quote::format_ident!("{}", rules.grammar_name);
    let memo_ctx_mod = quote::format_ident!("{}_impl_mod", rules.grammar_name);

    for (name, (node, node_ty)) in &rules.rules {
        let GeneratedRule{ parser_fn, memo_id, memo_ty } = generate_code_rule(rules, node_ty, name, node);

        parser_fns.push(parser_fn);

        memo_members.push(quote!{ #memo_id: HashMap<usize, #memo_ty> });
        memo_ctor.push(quote!{ #memo_id: HashMap::new() });
    }

    quote!{
        mod #memo_ctx_mod {
            use ::yk_parser::{irec, drec, ParseResult, ParseOk, ParseErr};
            use ::std::string::String;
            use ::std::option::Option;
            use ::std::collections::HashMap;
            use ::std::iter::Iterator;
            use ::std::clone::Clone;
            use ::std::cmp::{PartialEq, Eq};
            use ::std::hash::Hash;
            use ::std::boxed::Box;
            use ::std::rc::Rc;
            use ::std::cell::{RefCell, RefMut};
            // TODO: Is this temporary?
            use ::std::fmt::Display;

            pub struct #memo_ctx<I> {
                call_stack: irec::CallStack,
                call_heads: irec::CallHeadTable,
                #(#memo_members),*
            }

            impl <I> #memo_ctx<I> {
                pub fn new() -> Self {
                    Self{
                        call_stack: irec::CallStack::new(),
                        call_heads: irec::CallHeadTable::new(),
                        #(#memo_ctor),*
                    }
                }

                #(#parser_fns)*
            }

            // TODO: Probably something better?
            // Like a custom hash map wrapper for the memo context tables?
            fn insert_and_get<K, V>(m: &mut HashMap<K, V>, k: K, v: V) -> &V where K : Clone + Eq + Hash {
                m.insert(k.clone(), v);
                m.get(&k).unwrap()
            }
        }

        use #memo_ctx_mod::#memo_ctx;
    }
}

fn generate_code_rule(rs: &bnf::RuleSet, ret_ty: &Type,
    name: &str, node: &bnf::Node) -> GeneratedRule {

    // Generate code for the subrule
    let (code, counter) = generate_code_node(rs, 0, node);

    //let ret_tys: Vec<_> = (0..counter).map(|_| quote!{ i32 }).collect();
    //let ret_ty = quote!{ (#(#ret_tys),*) };
    let ret_ty = quote!{ #ret_ty };

    // Any function that wants to respect the same constraints as the parser will have to
    // have this where clause
    let where_clause = quote!{
        I : Iterator + Clone,
        <I as Iterator>::Item :
            // TODO: Collect what we compare with!
              PartialEq<char>

            + Display,

            // TODO: Do we need this everywhere?
            // Can we at least get rid of the iterator?
            I : 'static,
            #ret_ty : 'static
    };

    // Identifiers for this parser
    let pub_parse_fname = quote::format_ident!("{}", name);
    let parse_fname = quote::format_ident!("parse_{}", name);
    let grow_fname = quote::format_ident!("grow_{}", name);
    let recall_fname = quote::format_ident!("recall_{}", name);
    let lr_answer_fname = quote::format_ident!("lr_answer_{}", name);
    let memo_id = quote::format_ident!("memo_{}", name);
    let memo_ctx = quote::format_ident!("{}", rs.grammar_name);

    // How to reference the memo table's current entry
    let memo_entry = quote!{ self.#memo_id };

    let rec = rs.left_recursion(name);
    let memo_ty = match rec {
        bnf::LeftRecursion::None => {quote!{
            ParseResult<I, #ret_ty>
        }},

        bnf::LeftRecursion::Direct => {quote!{
            drec::DirectRec<I, #ret_ty>
        }},

        bnf::LeftRecursion::Indirect => {quote!{
            irec::Entry<I, #ret_ty>
        }}
    };

    let memo_code = match rec {
        bnf::LeftRecursion::None => {quote!{
            // TODO: Oof... We are cloning the result!
            if let Some(res) = #memo_entry.get(&idx) {
                res.clone()
            }
            else {
                let res = { #code };
                insert_and_get(&mut #memo_entry, idx, res).clone()
            }
        }},

        bnf::LeftRecursion::Direct => {quote!{
            // TODO: Oof... We are cloning the result!
            match #memo_entry.get(&idx) {
                None => {
                    // Nothing is in the cache, write a dummy error
                    #memo_entry.insert(idx, drec::DirectRec::Base(ParseErr::new().into()));
                    // Now invoke the parser
                    // If it's recursive, the entry must have changed
                    let tmp_res = { #code };
                    // Refresh the entry, check
                    match #memo_entry.get(&idx).unwrap() {
                        drec::DirectRec::Recurse(_) => {
                            // We are in recursion!
                            let old = insert_and_get(
                                &mut #memo_entry, idx, drec::DirectRec::Recurse(tmp_res)).parse_result().clone();
                            self.#grow_fname(src.clone(), idx, old)
                        },

                        drec::DirectRec::Base(_) => {
                            // No change, write back result, prevent recursion
                            insert_and_get(
                                &mut #memo_entry, idx, drec::DirectRec::Stub(tmp_res)).parse_result().clone()
                        },

                        _ => panic!("Unreachable!"),
                    }
                },

                Some(drec::DirectRec::Base(res)) => {
                    // Recursion signal, write back a dummy error to start!
                    // TODO: Instead of cloning we could just remove it from here!
                    insert_and_get(
                        &mut #memo_entry, idx, drec::DirectRec::Recurse(ParseErr::new().into())).parse_result().clone()
                },

                  Some(drec::DirectRec::Stub(res))
                | Some(drec::DirectRec::Recurse(res)) => {
                    res.clone()
                }
            }
        }},

        bnf::LeftRecursion::Indirect => {quote!{
            let m = self.#recall_fname(src.clone(), idx); // Option<irec::Entry<I, T>>

            match m {
                None => {
                    let mut base =
                        Rc::new(RefCell::new(irec::LeftRecursive::with_parser_and_seed::<I, #ret_ty>(#name, ParseErr::new().into())));
                    self.call_stack.push(base.clone());
                    #memo_entry.insert(idx, irec::Entry::LeftRecursive(base.clone()));
                    let tmp_res = { #code };
                    self.call_stack.pop();

                    if base.borrow().head.is_none() {
                        insert_and_get(&mut #memo_entry, idx, irec::Entry::ParseResult(tmp_res)).parse_result()
                    }
                    else {
                        base.borrow_mut().seed = Box::new(tmp_res);
                        self.#lr_answer_fname(src.clone(), idx, &base)
                    }
                },

                Some(irec::Entry::LeftRecursive(lr)) => {
                    self.call_stack.setup_lr(#name, &lr);
                    lr.borrow().parse_result()
                },

                Some(irec::Entry::ParseResult(r)) => {
                    r.clone()
                }
            }
        }}
    };

    // Generate extra functions
    let grow_code = match rec {
        bnf::LeftRecursion::None => quote!{},

        bnf::LeftRecursion::Direct => {quote!{
            fn #grow_fname(&mut self, src: I, idx: usize,
                old: ParseResult<I, #ret_ty>) -> ParseResult<I, #ret_ty>
                where #where_clause {

                let curr_rule = #name;

                if old.is_err() {
                    return old;
                }

                let tmp_res = { #code };
                // TODO: Oof, unnecessary cloning
                if tmp_res.is_ok() && old.furthest_look() < tmp_res.furthest_look() {
                    // Successfully grew the seed
                    let new_old = insert_and_get(
                        &mut #memo_entry, idx, drec::DirectRec::Recurse(tmp_res)).parse_result().clone();
                    return self.#grow_fname(src, idx, new_old);
                }

                // We need to overwrite max-furthest in the memo-table!
                // That's why we don't simply return old_res
                let updated = ParseResult::unify_alternatives(tmp_res, old);
                return insert_and_get(
                    &mut #memo_entry, idx, drec::DirectRec::Recurse(updated)).parse_result().clone();
            }
        }},

        bnf::LeftRecursion::Indirect => {quote!{
            fn #recall_fname(&mut self, src: I, idx: usize)
                -> Option<irec::Entry<I, #ret_ty>>
                where #where_clause {

                let curr_rule = #name;

                let cached = #memo_entry.get(&idx);
                let in_heads = self.call_heads.get(&idx);

                match (in_heads, cached) {
                    (None, None) => None,
                    (None, Some(c)) => Some((*c).clone()),

                    (Some(h), c) => {
                        if c.is_none() && !(#name == h.borrow().head || h.borrow().involved.contains(#name)) {
                            Some(irec::Entry::ParseResult(ParseErr::new().into()))
                        }
                        else if h.borrow_mut().eval.remove(#name) {
                            let tmp_res = { #code };
                            Some(insert_and_get(&mut #memo_entry, idx, irec::Entry::ParseResult(tmp_res)).clone())
                        }
                        else {
                            c.cloned()
                        }
                    }
                }
            }

            fn #lr_answer_fname(&mut self, src: I, idx: usize,
                growable: &Rc<RefCell<irec::LeftRecursive>>)
                -> ParseResult<I, #ret_ty>
                where #where_clause {

                assert!((*growable).borrow().head.is_some());

                let seed = (*growable).borrow().parse_result();

                if (*growable).borrow().head.as_ref().unwrap().borrow().head != #name {
                    return seed;
                }

                let s = insert_and_get(&mut #memo_entry, idx, irec::Entry::ParseResult(seed)).parse_result();
                if s.is_err() {
                    return s;
                }
                else {
                    return self.#grow_fname(src, idx, s, (*growable).borrow().head.as_ref().unwrap());
                }
            }

            fn #grow_fname(&mut self, src: I, idx: usize,
                old: ParseResult<I, #ret_ty>, h: &Rc<RefCell<irec::RecursionHead>>)
                -> ParseResult<I, #ret_ty>
                where #where_clause {

                let curr_rule = #name;

                self.call_heads.insert(idx, h.clone());
                let involved_clone = (*h).borrow().involved.clone();
                h.borrow_mut().eval = involved_clone;

                let tmp_res = { #code };
                // TODO: Oof, unnecessary cloning
                if tmp_res.is_ok() && old.furthest_look() < tmp_res.furthest_look() {
                    // Successfully grew the seed
                    let new_old = insert_and_get(
                        &mut #memo_entry, idx, irec::Entry::ParseResult(tmp_res)).parse_result();
                    return self.#grow_fname(src, idx, new_old, h);
                }

                // We need to overwrite max-furthest in the memo-table!
                // That's why we don't simply return old_res
                self.call_heads.remove(&idx);
                let updated = ParseResult::unify_alternatives(tmp_res, old);
                return insert_and_get(
                    &mut #memo_entry, idx, irec::Entry::ParseResult(updated)).parse_result();
            }
        }},
    };

    let parser_fn = quote!{
        #grow_code

        // The actual published function
        pub fn #pub_parse_fname(&mut self, src: I) -> ParseResult<I, #ret_ty> where #where_clause {
            self.#parse_fname(src, 0)
        }

        fn #parse_fname(&mut self, src: I, idx: usize) -> ParseResult<I, #ret_ty> where #where_clause {
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
                self.#fname(src.clone(), idx)
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
