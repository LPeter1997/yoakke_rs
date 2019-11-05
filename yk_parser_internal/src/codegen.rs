/**
 * Code generation from the BNF AST.
 */

use std::collections::HashMap;
use proc_macro2::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{Ident, Block, Lit, Path, Type};
use syn::token::Brace;
use crate::bnf;
use crate::parse_result::*;
use crate::replace_dollar::replace_dollar;

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

    let mut memo_invalidate = Vec::new();

    // Identifier for type
    //let memo_ctx = quote::format_ident!("{}", rules.grammar_name);
    // memo_ctx_mod = quote::format_ident!("{}_impl_mod", rules.grammar_name);

    for (name, (node, node_ty)) in &rules.rules {
        let GeneratedRule{ parser_fn, memo_id, memo_ty } = generate_code_rule(rules, node_ty, name, node);

        parser_fns.push(parser_fn);

        memo_members.push(quote!{ #memo_id: HashMap<usize, #memo_ty> });
        memo_ctor.push(quote!{ #memo_id: HashMap::new() });

        memo_invalidate.push(quote!{{
            // TODO: There's probably a better data-structure to do this
            // This is essentially a linear search on a hash-map...

            let mut entries_to_erase = Vec::new();
            let mut entries_to_move = Vec::new();

            // Collect what to remove and what to offset
            for (k, v) in &self.#memo_id {
                let start = *k;
                let end = start + v.furthest_look();
                if start >= rem.start {
                    if start < rem.end {
                        entries_to_erase.push(*k);
                    }
                    else {
                        entries_to_move.push(*k);
                    }
                }
                else if end > rem.start {
                    entries_to_erase.push(start);
                }
            }

            // Remove what needs to be removed
            for k in entries_to_erase {
                self.#memo_id.remove(&k);
            }

            // Offset what needs to be offset
            for k in entries_to_move {
                let new_k = usize::try_from(isize::try_from(k).unwrap() + offset).unwrap();
                let entry = self.#memo_id.remove(&k).unwrap();
                self.#memo_id.insert(new_k, entry);
            }
        }});
    }

    let item_type = &rules.item_type;

    quote!{
        //mod #memo_ctx_mod {
            use ::yk_parser::{irec, drec, ParseResult, ParseOk, ParseErr, Found, Match, EndOfInput};
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
            use ::std::ops::Range;
            use ::std::convert::TryFrom;

            pub struct Parser {
                call_stack: irec::CallStack,
                call_heads: irec::CallHeadTable,
                #(#memo_members),*
            }

            impl ::yk_parser::Parser for Parser {
                type Item = #item_type;
            }

            impl Parser {
                pub fn new() -> Self {
                    Self{
                        call_stack: irec::CallStack::new(),
                        call_heads: irec::CallHeadTable::new(),
                        #(#memo_ctor),*
                    }
                }

                pub fn invalidate(&mut self, rem: Range<usize>, ins: usize) {
                    let offset = isize::try_from(ins).unwrap() - isize::try_from(rem.end - rem.start).unwrap();

                    // TODO: Invalidate call_stack and call_heads too

                    #(#memo_invalidate)*
                }

                #(#parser_fns)*
            }

            // TODO: Probably something better?
            // Like a custom hash map wrapper for the memo context tables?
            fn insert_and_get<K, V>(m: &mut HashMap<K, V>, k: K, v: V) -> &V where K : Clone + Eq + Hash {
                m.insert(k.clone(), v);
                m.get(&k).unwrap()
            }
        //}

        //use #memo_ctx_mod::#memo_ctx;
    }
}

fn generate_code_rule(rs: &bnf::RuleSet, ret_ty: &Type,
    name: &str, node: &bnf::Node) -> GeneratedRule {

    // Generate code for the subrule
    let (code, counter) = generate_code_node(rs, ret_ty, 0, node);

    //let ret_tys: Vec<_> = (0..counter).map(|_| quote!{ i32 }).collect();
    //let ret_ty = quote!{ (#(#ret_tys),*) };
    let ret_ty = quote!{ #ret_ty };

    // Any function that wants to respect the same constraints as the parser will have to
    // have this where clause
    let item_type = &rs.item_type;
    let where_clause = quote!{
        I : Iterator<Item = #item_type> + Clone,

        // TODO: Do we need this everywhere?
        // Maybe enough for indirect recursion
        #ret_ty : 'static,
        #item_type : 'static
    };

    // Identifiers for this parser
    let pub_parse_fname = quote::format_ident!("{}", name);
    let parse_fname     = quote::format_ident!("parse_{}", name);
    let apply_fname     = quote::format_ident!("apply_{}", name);
    let grow_fname      = quote::format_ident!("grow_{}", name);
    let recall_fname    = quote::format_ident!("recall_{}", name);
    let lr_answer_fname = quote::format_ident!("lr_answer_{}", name);
    let memo_id         = quote::format_ident!("memo_{}", name);
    //let memo_ctx = quote::format_ident!("{}", rs.grammar_name);

    // How to reference the memo table's current entry
    let memo_entry = quote!{ self.#memo_id };

    let rec = rs.left_recursion(name);
    let memo_ty = match rec {
        bnf::LeftRecursion::None => {quote!{
            ParseResult<#ret_ty, #item_type>
        }},

        bnf::LeftRecursion::Direct => {quote!{
            drec::DirectRec<#ret_ty, #item_type>
        }},

        bnf::LeftRecursion::Indirect => {quote!{
            irec::Entry<#ret_ty, #item_type>
        }}
    };

    let memo_code = match rec {
        bnf::LeftRecursion::None => {quote!{
            // TODO: Oof... We are cloning the result!
            if let Some(res) = #memo_entry.get(&idx) {
                res.clone()
            }
            else {
                let res = self.#apply_fname(src.clone(), idx);
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
                    let tmp_res = self.#apply_fname(src.clone(), idx);
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
            let m = self.#recall_fname(src.clone(), idx); // Option<irec::Entry<T>>

            match m {
                None => {
                    let mut base = Rc::new(RefCell::new(
                        irec::LeftRecursive::with_parser_and_seed::<#ret_ty, #item_type>(#name, ParseErr::new().into())));
                    self.call_stack.push(base.clone());
                    #memo_entry.insert(idx, irec::Entry::LeftRecursive(base.clone()));
                    let tmp_res = self.#apply_fname(src.clone(), idx);
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
            fn #grow_fname<I>(&mut self, src: I, idx: usize,
                old: ParseResult<#ret_ty, #item_type>) -> ParseResult<#ret_ty, #item_type>
                where #where_clause {

                let curr_rule = #name;

                if old.is_err() {
                    return old;
                }

                let tmp_res = self.#apply_fname(src.clone(), idx);
                // TODO: Oof, unnecessary cloning
                if tmp_res.is_ok() && old.furthest_look() < tmp_res.furthest_look() {
                    // Successfully grew the seed
                    let new_old = insert_and_get(
                        &mut #memo_entry, idx, drec::DirectRec::Recurse(tmp_res)).parse_result().clone();
                    return self.#grow_fname(src.clone(), idx, new_old);
                }

                // We need to overwrite max-furthest in the memo-table!
                // That's why we don't simply return old_res
                let updated = ParseResult::unify_alternatives(tmp_res, old);
                return insert_and_get(
                    &mut #memo_entry, idx, drec::DirectRec::Recurse(updated)).parse_result().clone();
            }
        }},

        bnf::LeftRecursion::Indirect => {quote!{
            fn #recall_fname<I>(&mut self, src: I, idx: usize)
                -> Option<irec::Entry<#ret_ty, #item_type>>
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
                            let tmp_res = self.#apply_fname(src.clone(), idx);
                            Some(insert_and_get(&mut #memo_entry, idx, irec::Entry::ParseResult(tmp_res)).clone())
                        }
                        else {
                            c.cloned()
                        }
                    }
                }
            }

            fn #lr_answer_fname<I>(&mut self, src: I, idx: usize,
                growable: &Rc<RefCell<irec::LeftRecursive>>)
                -> ParseResult<#ret_ty, #item_type>
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
                    return self.#grow_fname(src.clone(), idx, s, (*growable).borrow().head.as_ref().unwrap());
                }
            }

            fn #grow_fname<I>(&mut self, src: I, idx: usize,
                old: ParseResult<#ret_ty, #item_type>, h: &Rc<RefCell<irec::RecursionHead>>)
                -> ParseResult<#ret_ty, #item_type>
                where #where_clause {

                let curr_rule = #name;

                self.call_heads.insert(idx, h.clone());
                let involved_clone = (*h).borrow().involved.clone();
                h.borrow_mut().eval = involved_clone;

                let tmp_res = self.#apply_fname(src.clone(), idx);
                // TODO: Oof, unnecessary cloning
                if tmp_res.is_ok() && old.furthest_look() < tmp_res.furthest_look() {
                    // Successfully grew the seed
                    let new_old = insert_and_get(
                        &mut #memo_entry, idx, irec::Entry::ParseResult(tmp_res)).parse_result();
                    return self.#grow_fname(src.clone(), idx, new_old, h);
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
        pub fn #pub_parse_fname<I>(&mut self, src: I) -> ParseResult<#ret_ty, #item_type> where #where_clause {
            self.#parse_fname(src, 0)
        }

        fn #parse_fname<I>(&mut self, src: I, idx: usize) -> ParseResult<#ret_ty, #item_type> where #where_clause {
            let curr_rule = #name;
            #memo_code
        }

        fn #apply_fname<I>(&mut self, src: I, idx: usize) -> ParseResult<#ret_ty, #item_type> where #where_clause {
            let curr_rule = #name;
            // Enforce a cast if needed
            let res: ParseResult<_, #item_type> = { #code };
            res.map(|ok| -> #ret_ty { ok.into() })
            /*
            Without implicit cast:
            { #code }
            */
        }
    };

    GeneratedRule{ parser_fn, memo_id, memo_ty }
}

fn generate_code_node(rs: &bnf::RuleSet, ret_ty: &Type, counter: usize,
    node: &bnf::Node) -> (TokenStream, Vec<Type>) {

    match node {
        bnf::Node::Transformation{ subnode, action, } =>
            generate_code_transformation(rs, ret_ty, counter, subnode, action),

        bnf::Node::Alternative{ first, second, } =>
            generate_code_alternative(rs, ret_ty, counter, first, second),

        bnf::Node::Sequence{ first, second, } =>
            generate_code_sequence(rs, ret_ty, counter, first, second),

        bnf::Node::Literal(lit) => match lit {
            bnf::LiteralNode::Ident(p) => generate_code_ident(rs, counter, p),
            bnf::LiteralNode::Lit(l) => generate_code_lit(rs, counter, l),
            bnf::LiteralNode::Eps => generate_code_eps(rs, counter),
            bnf::LiteralNode::End => generate_code_end(rs, counter),
        },
    }
}

fn generate_code_transformation(rs: &bnf::RuleSet, ret_ty: &Type, counter: usize,
    node: &bnf::Node, (_, body): &(Brace, TokenStream)) -> (TokenStream, Vec<Type>) {

    assert_eq!(counter, 0);

    let (code, types) = generate_code_node(rs, ret_ty, counter, node);

    let params: Vec<_> = types.iter().enumerate().map(|(i, ty)| {
        let ident = quote::format_ident!("yk_param_{}", i);
        quote!{ #ident: #ty }
    }).collect();
    let params = quote!{ #(#params),* };
    let param_tys: Vec<_> = types.iter().map(|ty| quote!{ #ty }).collect();
    let param_tys = quote!{ #(#param_tys),* };

    // We need to create a Block from the passed in brace and token stream
    // First we need to substitute every $<integer> to a proper identifier
    // Create a substitution map
    let subst_map: HashMap<_, _> = (0..types.len()).enumerate()
        .map(|(i, _)| (i, format!("yk_param_{}", i)))
        .collect();
    let body = replace_dollar(body.clone(), &subst_map);
    let body = quote!{ { #body } };
    let action: Block = syn::parse2(body).unwrap();

    ///////////////////////////////////////////////////////////////////////

    let param_names = param_list(0..types.len());
    // Enforce a cast
    let closure = quote!{ |#params| -> #ret_ty { #action.into() } };
    /*
    Without cast:
    let closure = quote!{ |#params| -> #ret_ty #action };
    */

    let item_type = &rs.item_type;
    let code = quote!{{
        let res: ParseResult<_, #item_type> = { #code };
        res.map(|(#param_names): (#param_tys)| (#closure)(#param_names)).into()
    }};

    (code, vec![ret_ty.clone()])
}

fn generate_code_alternative(rs: &bnf::RuleSet, ret_ty: &Type, counter: usize,
    first: &bnf::Node, second: &bnf::Node) -> (TokenStream, Vec<Type>) {

    let (code1, types1) = generate_code_node(rs, ret_ty, counter, first);
    let (code2, types2) = generate_code_node(rs, ret_ty, counter, second);

    // Also they should be the same types
    assert_eq!(types1.len(), types2.len());

    let params = param_list(counter..(counter + types1.len()));

    let code = quote!{{
        let res1 = { #code1 };
        let res2 = { #code2 };
        ParseResult::unify_alternatives(res1, res2)
    }};

    (code, types1)
}

fn generate_code_sequence(rs: &bnf::RuleSet, ret_ty: &Type, counter: usize,
    first: &bnf::Node, second: &bnf::Node) -> (TokenStream, Vec<Type>) {

    let (code1, mut types1) = generate_code_node(rs, ret_ty, counter, first);
    let (code2, mut types2) = generate_code_node(rs, ret_ty, types1.len(), second);

    let params1 = param_list(counter..(counter + types1.len()));
    let params2 = param_list((counter + types1.len())..(counter + types1.len() + types2.len()));

    let code = quote!{{
        let res1 = { #code1 };
        if let ParseResult::Ok(ok) = res1 {
            // Overwrite positional data for the next part's invocation
            let src = {
                let mut src = src.clone();
                if ok.matched > 0 {
                    src.nth(ok.matched - 1);
                }
                src
            };
            let idx = idx + ok.matched;
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

    types1.append(&mut types2);
    (code, types1)
}

fn generate_code_lit(rs: &bnf::RuleSet, counter: usize, lit: &Lit) -> (TokenStream, Vec<Type>) {
    generate_code_atom(rs, counter, quote!{ #lit })
}

fn generate_code_ident(rs: &bnf::RuleSet, counter: usize, lit: &Path) -> (TokenStream, Vec<Type>) {

    if lit.leading_colon.is_none() && lit.segments.len() == 1 {
        let id = lit.segments[0].ident.to_string();

        if let Some((_, ty)) = rs.rules.get(&id) {
            // Rule identifier
            let fname = quote::format_ident!("parse_{}", id);
            let code = quote!{{
                let mut tmp_res = self.#fname(src.clone(), idx);
                // If there is an error but the furthest lookahead is one, the context can change
                // to the current rule to make it clearer
                match &mut tmp_res {
                    ParseResult::Ok(ok) => {
                        let mut next_err = None;
                        if let Some(err) = &mut ok.furthest_error {
                            if err.furthest_look == 1 {
                                //err.merge_element_into(#id, curr_rule);
                                next_err =
                                    Some(ParseErr::single(1, err.found_element.clone(), curr_rule, #id.into()));
                            }
                        }
                        if next_err.is_some() {
                            ok.furthest_error = next_err;
                        }
                    },

                    ParseResult::Err(err) => {
                        if err.furthest_look == 1 {
                            *err = ParseErr::single(1, err.found_element.clone(), curr_rule, #id.into());
                        }
                    }
                }
                tmp_res
            }};
            return (code, vec![ty.clone()]);
        }
    }

    // Some identifier
    return generate_code_atom(rs, counter, quote!{ #lit });
}

fn generate_code_atom(rs: &bnf::RuleSet, counter: usize, tok: TokenStream) -> (TokenStream, Vec<Type>) {
    let code = quote!{{
        let mut src2 = src.clone();
        if let Some(v) = src2.next() {
            if Self::matches(&v, &#tok) {
                ParseOk{ matched: 1, furthest_error: None, value: (v) }.into()
            }
            else {
                ParseErr::single(1, Found::Element(v), curr_rule, Self::show_expected(&#tok)).into()
            }
        }
        else {
            ParseErr::single(1, Found::EndOfInput, curr_rule, Self::show_expected(&#tok)).into()
        }
    }};
    (code, vec![rs.item_type.clone()])
}

fn generate_code_end(rs: &bnf::RuleSet, counter: usize) -> (TokenStream, Vec<Type>) {
    let unit_ty = Type::Tuple(syn::TypeTuple{
        paren_token: syn::token::Paren{ span: proc_macro2::Span::call_site() },
        elems: syn::punctuated::Punctuated::new()
    });
    let code = quote!{{
        let mut src2 = src.clone();
        if let Some(v) = src2.next() {
            let eoi = EndOfInput{};
            if Self::matches(&v, &eoi) {
                ParseOk{ matched: 1, furthest_error: None, value: () }.into()
            }
            else {
                ParseErr::single(1, Found::Element(v), curr_rule, Self::show_expected(&eoi)).into()
            }
        }
        else {
            ParseOk{ matched: 0, furthest_error: None, value: () }.into()
        }
    }};
    (code, vec![unit_ty])
}

fn generate_code_eps(rs: &bnf::RuleSet, counter: usize) -> (TokenStream, Vec<Type>) {
    // Empty aceptance
    let code = quote!{
        ParseOk{ matched: 0, furthest_error: None, value: () }.into()
    };
    let unit_ty = Type::Tuple(syn::TypeTuple{
        paren_token: syn::token::Paren{ span: proc_macro2::Span::call_site() },
        elems: syn::punctuated::Punctuated::new()
    });
    return (code, vec![unit_ty]);
}

// Helpers

fn param_list(r: std::ops::Range<usize>) -> TokenStream {
    let params: Vec<_> = r.map(|x| quote::format_ident!("yk_param_{}", x)).collect();
    quote!{ #(#params),* }
}
