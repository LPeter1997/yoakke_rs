
extern crate yk_lexer;
extern crate yk_parser;

use yk_lexer::{Token, TokenType, Lexer};
use yk_parser::{yk_parser, ParseResult, Match};

#[derive(Lexer, Clone, Copy, PartialEq, Eq, Debug)]
enum TokTy {
    #[error] Error,
    #[end] End,
    #[regex(r"[ \r\n]")] #[ignore] Whitespace,

    #[c_ident] Ident, // Unused in example, just for demonstration
    #[regex("[0-9]+")] IntLit,

    #[token("+")] Add,
    #[token("-")] Sub,
    #[token("*")] Mul,
    #[token("/")] Div,

    #[token(">")] Gr,
    #[token(">=")] GrEq,
    #[token("<")] Le,
    #[token("<=")] LeEq,

    #[token("==")] Eq,
    #[token("!=")] Neq,

    #[token("(")] LeftParen,
    #[token(")")] RightParen,
}

mod peg {
    use crate::TokTy;
    use yk_parser::yk_parser;
    use yk_lexer::Token;

    use std::boxed::Box;
use std::cell::{RefCell, RefMut};
use std::clone::Clone;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Iterator;
use std::option::Option;
use std::rc::Rc;
use std::string::String;
use yk_parser::{drec, irec, Match, ParseErr, ParseOk, ParseResult, ShowExpected, ShowFound};
pub struct Parser<I> {
    call_stack: irec::CallStack,
    call_heads: irec::CallHeadTable,
    memo_expr: HashMap<usize, ParseResult<I, i32>>,
    memo_add_expr: HashMap<usize, drec::DirectRec<I, i32>>,
    memo_mul_expr: HashMap<usize, drec::DirectRec<I, i32>>,
    memo_atom: HashMap<usize, ParseResult<I, i32>>,
    memo_rel_expr: HashMap<usize, drec::DirectRec<I, i32>>,
    memo_eqty_expr: HashMap<usize, drec::DirectRec<I, i32>>,
}
impl<I> ::yk_parser::Parser for Parser<I>
where
    I: Iterator<Item = Token<TokTy>>,
{
    type Item = Token<TokTy>;
}
impl<I> Parser<I>
where
    I: Iterator<Item = Token<TokTy>>,
{
    pub fn new() -> Self {
        Self {
            call_stack: irec::CallStack::new(),
            call_heads: irec::CallHeadTable::new(),
            memo_expr: HashMap::new(),
            memo_add_expr: HashMap::new(),
            memo_mul_expr: HashMap::new(),
            memo_atom: HashMap::new(),
            memo_rel_expr: HashMap::new(),
            memo_eqty_expr: HashMap::new(),
        }
    }
    pub fn expr(&mut self, src: I) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        self.parse_expr(src, 0)
    }
    fn parse_expr(&mut self, src: I, idx: usize) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        let curr_rule = "expr";
        if let Some(res) = self.memo_expr.get(&idx) {
            res.clone()
        } else {
            let res = { self.parse_eqty_expr(src.clone(), idx) };
            insert_and_get(&mut self.memo_expr, idx, res).clone()
        }
    }
    fn grow_add_expr(&mut self, src: I, idx: usize, old: ParseResult<I, i32>) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        let curr_rule = "add_expr";
        if old.is_err() {
            return old;
        }
        let tmp_res = {
            {
                let res1 = {
                    {
                        let res = {
                            {
                                let res1 = { self.parse_add_expr(src.clone(), idx) };
                                if let ParseResult::Ok(ok) = res1 {
                                    let src = ok.furthest_it.clone();
                                    let idx = ok.matched;
                                    let res2 = {
                                        {
                                            let res1 = {
                                                {
                                                    let mut src2 = src.clone();
                                                    if let Some(v) = src2.next() {
                                                        if Self::matches(&v, &TokTy::Add) {
                                                            ParseOk {
                                                                matched: idx + 1,
                                                                furthest_it: src2,
                                                                furthest_error: None,
                                                                value: (v),
                                                            }
                                                            .into()
                                                        } else {
                                                            let got = Self::show_found(&v);
                                                            ParseErr::single(
                                                                idx,
                                                                got,
                                                                curr_rule,
                                                                Self::show_expected(&TokTy::Add),
                                                            )
                                                            .into()
                                                        }
                                                    } else {
                                                        ParseErr::single(
                                                            idx,
                                                            "end of input".into(),
                                                            curr_rule,
                                                            Self::show_expected(&TokTy::Add),
                                                        )
                                                        .into()
                                                    }
                                                }
                                            };
                                            if let ParseResult::Ok(ok) = res1 {
                                                let src = ok.furthest_it.clone();
                                                let idx = ok.matched;
                                                let res2 =
                                                    { self.parse_mul_expr(src.clone(), idx) };
                                                let res_tmp = ParseResult::unify_sequence(ok, res2);
                                                if let ParseResult::Ok(ok) = res_tmp {
                                                    ok.map(|((e1), (e2))| (e1, e2)).into()
                                                } else {
                                                    res_tmp.err().unwrap().into()
                                                }
                                            } else {
                                                res1.err().unwrap().into()
                                            }
                                        }
                                    };
                                    let res_tmp = ParseResult::unify_sequence(ok, res2);
                                    if let ParseResult::Ok(ok) = res_tmp {
                                        ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                    } else {
                                        res_tmp.err().unwrap().into()
                                    }
                                } else {
                                    res1.err().unwrap().into()
                                }
                            }
                        };
                        if let ParseResult::Ok(ok) = res {
                            ok.map(|(e0, e1, e2)| (|e0, e1, e2| e0 + e2)(e0, e1, e2))
                                .into()
                        } else {
                            res.err().unwrap().into()
                        }
                    }
                };
                let res2 = {
                    {
                        let res1 = {
                            {
                                let res = {
                                    {
                                        let res1 = { self.parse_add_expr(src.clone(), idx) };
                                        if let ParseResult::Ok(ok) = res1 {
                                            let src = ok.furthest_it.clone();
                                            let idx = ok.matched;
                                            let res2 = {
                                                {
                                                    let res1 = {
                                                        {
                                                            let mut src2 = src.clone();
                                                            if let Some(v) = src2.next() {
                                                                if Self::matches(&v, &TokTy::Sub) {
                                                                    ParseOk {
                                                                        matched: idx + 1,
                                                                        furthest_it: src2,
                                                                        furthest_error: None,
                                                                        value: (v),
                                                                    }
                                                                    .into()
                                                                } else {
                                                                    let got = Self::show_found(&v);
                                                                    ParseErr::single(
                                                                        idx,
                                                                        got,
                                                                        curr_rule,
                                                                        Self::show_expected(
                                                                            &TokTy::Sub,
                                                                        ),
                                                                    )
                                                                    .into()
                                                                }
                                                            } else {
                                                                ParseErr::single(
                                                                    idx,
                                                                    "end of input".into(),
                                                                    curr_rule,
                                                                    Self::show_expected(
                                                                        &TokTy::Sub,
                                                                    ),
                                                                )
                                                                .into()
                                                            }
                                                        }
                                                    };
                                                    if let ParseResult::Ok(ok) = res1 {
                                                        let src = ok.furthest_it.clone();
                                                        let idx = ok.matched;
                                                        let res2 = {
                                                            self.parse_mul_expr(src.clone(), idx)
                                                        };
                                                        let res_tmp =
                                                            ParseResult::unify_sequence(ok, res2);
                                                        if let ParseResult::Ok(ok) = res_tmp {
                                                            ok.map(|((e1), (e2))| (e1, e2)).into()
                                                        } else {
                                                            res_tmp.err().unwrap().into()
                                                        }
                                                    } else {
                                                        res1.err().unwrap().into()
                                                    }
                                                }
                                            };
                                            let res_tmp = ParseResult::unify_sequence(ok, res2);
                                            if let ParseResult::Ok(ok) = res_tmp {
                                                ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                            } else {
                                                res_tmp.err().unwrap().into()
                                            }
                                        } else {
                                            res1.err().unwrap().into()
                                        }
                                    }
                                };
                                if let ParseResult::Ok(ok) = res {
                                    ok.map(|(e0, e1, e2)| (|e0, e1, e2| e0 - e2)(e0, e1, e2))
                                        .into()
                                } else {
                                    res.err().unwrap().into()
                                }
                            }
                        };
                        let res2 = { self.parse_mul_expr(src.clone(), idx) };
                        ParseResult::unify_alternatives(res1, res2)
                    }
                };
                ParseResult::unify_alternatives(res1, res2)
            }
        };
        if tmp_res.is_ok() && old.furthest_look() < tmp_res.furthest_look() {
            let new_old = insert_and_get(
                &mut self.memo_add_expr,
                idx,
                drec::DirectRec::Recurse(tmp_res),
            )
            .parse_result()
            .clone();
            return self.grow_add_expr(src, idx, new_old);
        }
        let updated = ParseResult::unify_alternatives(tmp_res, old);
        return insert_and_get(
            &mut self.memo_add_expr,
            idx,
            drec::DirectRec::Recurse(updated),
        )
        .parse_result()
        .clone();
    }
    pub fn add_expr(&mut self, src: I) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        self.parse_add_expr(src, 0)
    }
    fn parse_add_expr(&mut self, src: I, idx: usize) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        let curr_rule = "add_expr";
        match self.memo_add_expr.get(&idx) {
            None => {
                self.memo_add_expr
                    .insert(idx, drec::DirectRec::Base(ParseErr::new().into()));
                let tmp_res = {
                    {
                        let res1 = {
                            {
                                let res = {
                                    {
                                        let res1 = { self.parse_add_expr(src.clone(), idx) };
                                        if let ParseResult::Ok(ok) = res1 {
                                            let src = ok.furthest_it.clone();
                                            let idx = ok.matched;
                                            let res2 = {
                                                {
                                                    let res1 = {
                                                        {
                                                            let mut src2 = src.clone();
                                                            if let Some(v) = src2.next() {
                                                                if Self::matches(&v, &TokTy::Add) {
                                                                    ParseOk {
                                                                        matched: idx + 1,
                                                                        furthest_it: src2,
                                                                        furthest_error: None,
                                                                        value: (v),
                                                                    }
                                                                    .into()
                                                                } else {
                                                                    let got = Self::show_found(&v);
                                                                    ParseErr::single(
                                                                        idx,
                                                                        got,
                                                                        curr_rule,
                                                                        Self::show_expected(
                                                                            &TokTy::Add,
                                                                        ),
                                                                    )
                                                                    .into()
                                                                }
                                                            } else {
                                                                ParseErr::single(
                                                                    idx,
                                                                    "end of input".into(),
                                                                    curr_rule,
                                                                    Self::show_expected(
                                                                        &TokTy::Add,
                                                                    ),
                                                                )
                                                                .into()
                                                            }
                                                        }
                                                    };
                                                    if let ParseResult::Ok(ok) = res1 {
                                                        let src = ok.furthest_it.clone();
                                                        let idx = ok.matched;
                                                        let res2 = {
                                                            self.parse_mul_expr(src.clone(), idx)
                                                        };
                                                        let res_tmp =
                                                            ParseResult::unify_sequence(ok, res2);
                                                        if let ParseResult::Ok(ok) = res_tmp {
                                                            ok.map(|((e1), (e2))| (e1, e2)).into()
                                                        } else {
                                                            res_tmp.err().unwrap().into()
                                                        }
                                                    } else {
                                                        res1.err().unwrap().into()
                                                    }
                                                }
                                            };
                                            let res_tmp = ParseResult::unify_sequence(ok, res2);
                                            if let ParseResult::Ok(ok) = res_tmp {
                                                ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                            } else {
                                                res_tmp.err().unwrap().into()
                                            }
                                        } else {
                                            res1.err().unwrap().into()
                                        }
                                    }
                                };
                                if let ParseResult::Ok(ok) = res {
                                    ok.map(|(e0, e1, e2)| (|e0, e1, e2| e0 + e2)(e0, e1, e2))
                                        .into()
                                } else {
                                    res.err().unwrap().into()
                                }
                            }
                        };
                        let res2 = {
                            {
                                let res1 = {
                                    {
                                        let res = {
                                            {
                                                let res1 =
                                                    { self.parse_add_expr(src.clone(), idx) };
                                                if let ParseResult::Ok(ok) = res1 {
                                                    let src = ok.furthest_it.clone();
                                                    let idx = ok.matched;
                                                    let res2 = {
                                                        {
                                                            let res1 = {
                                                                {
                                                                    let mut src2 = src.clone();
                                                                    if let Some(v) = src2.next() {
                                                                        if Self::matches(
                                                                            &v,
                                                                            &TokTy::Sub,
                                                                        ) {
                                                                            ParseOk {
                                                                                matched: idx + 1,
                                                                                furthest_it: src2,
                                                                                furthest_error:
                                                                                    None,
                                                                                value: (v),
                                                                            }
                                                                            .into()
                                                                        } else {
                                                                            let got =
                                                                                Self::show_found(
                                                                                    &v,
                                                                                );
                                                                            ParseErr::single(
                                                                                idx,
                                                                                got,
                                                                                curr_rule,
                                                                                Self::show_expected(
                                                                                    &TokTy::Sub,
                                                                                ),
                                                                            )
                                                                            .into()
                                                                        }
                                                                    } else {
                                                                        ParseErr::single(
                                                                            idx,
                                                                            "end of input".into(),
                                                                            curr_rule,
                                                                            Self::show_expected(
                                                                                &TokTy::Sub,
                                                                            ),
                                                                        )
                                                                        .into()
                                                                    }
                                                                }
                                                            };
                                                            if let ParseResult::Ok(ok) = res1 {
                                                                let src = ok.furthest_it.clone();
                                                                let idx = ok.matched;
                                                                let res2 = {
                                                                    self.parse_mul_expr(
                                                                        src.clone(),
                                                                        idx,
                                                                    )
                                                                };
                                                                let res_tmp =
                                                                    ParseResult::unify_sequence(
                                                                        ok, res2,
                                                                    );
                                                                if let ParseResult::Ok(ok) = res_tmp
                                                                {
                                                                    ok.map(|((e1), (e2))| (e1, e2))
                                                                        .into()
                                                                } else {
                                                                    res_tmp.err().unwrap().into()
                                                                }
                                                            } else {
                                                                res1.err().unwrap().into()
                                                            }
                                                        }
                                                    };
                                                    let res_tmp =
                                                        ParseResult::unify_sequence(ok, res2);
                                                    if let ParseResult::Ok(ok) = res_tmp {
                                                        ok.map(|((e0), (e1, e2))| (e0, e1, e2))
                                                            .into()
                                                    } else {
                                                        res_tmp.err().unwrap().into()
                                                    }
                                                } else {
                                                    res1.err().unwrap().into()
                                                }
                                            }
                                        };
                                        if let ParseResult::Ok(ok) = res {
                                            ok.map(|(e0, e1, e2)| {
                                                (|e0, e1, e2| e0 - e2)(e0, e1, e2)
                                            })
                                            .into()
                                        } else {
                                            res.err().unwrap().into()
                                        }
                                    }
                                };
                                let res2 = { self.parse_mul_expr(src.clone(), idx) };
                                ParseResult::unify_alternatives(res1, res2)
                            }
                        };
                        ParseResult::unify_alternatives(res1, res2)
                    }
                };
                match self.memo_add_expr.get(&idx).unwrap() {
                    drec::DirectRec::Recurse(_) => {
                        let old = insert_and_get(
                            &mut self.memo_add_expr,
                            idx,
                            drec::DirectRec::Recurse(tmp_res),
                        )
                        .parse_result()
                        .clone();
                        self.grow_add_expr(src.clone(), idx, old)
                    }
                    drec::DirectRec::Base(_) => {
                        insert_and_get(&mut self.memo_add_expr, idx, drec::DirectRec::Stub(tmp_res))
                            .parse_result()
                            .clone()
                    }
                    _ => panic!("Unreachable!"),
                }
            }
            Some(drec::DirectRec::Base(res)) => insert_and_get(
                &mut self.memo_add_expr,
                idx,
                drec::DirectRec::Recurse(ParseErr::new().into()),
            )
            .parse_result()
            .clone(),
            Some(drec::DirectRec::Stub(res)) | Some(drec::DirectRec::Recurse(res)) => res.clone(),
        }
    }
    fn grow_mul_expr(&mut self, src: I, idx: usize, old: ParseResult<I, i32>) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        let curr_rule = "mul_expr";
        if old.is_err() {
            return old;
        }
        let tmp_res = {
            {
                let res1 = {
                    {
                        let res = {
                            {
                                let res1 = { self.parse_mul_expr(src.clone(), idx) };
                                if let ParseResult::Ok(ok) = res1 {
                                    let src = ok.furthest_it.clone();
                                    let idx = ok.matched;
                                    let res2 = {
                                        {
                                            let res1 = {
                                                {
                                                    let mut src2 = src.clone();
                                                    if let Some(v) = src2.next() {
                                                        if Self::matches(&v, &TokTy::Mul) {
                                                            ParseOk {
                                                                matched: idx + 1,
                                                                furthest_it: src2,
                                                                furthest_error: None,
                                                                value: (v),
                                                            }
                                                            .into()
                                                        } else {
                                                            let got = Self::show_found(&v);
                                                            ParseErr::single(
                                                                idx,
                                                                got,
                                                                curr_rule,
                                                                Self::show_expected(&TokTy::Mul),
                                                            )
                                                            .into()
                                                        }
                                                    } else {
                                                        ParseErr::single(
                                                            idx,
                                                            "end of input".into(),
                                                            curr_rule,
                                                            Self::show_expected(&TokTy::Mul),
                                                        )
                                                        .into()
                                                    }
                                                }
                                            };
                                            if let ParseResult::Ok(ok) = res1 {
                                                let src = ok.furthest_it.clone();
                                                let idx = ok.matched;
                                                let res2 = { self.parse_atom(src.clone(), idx) };
                                                let res_tmp = ParseResult::unify_sequence(ok, res2);
                                                if let ParseResult::Ok(ok) = res_tmp {
                                                    ok.map(|((e1), (e2))| (e1, e2)).into()
                                                } else {
                                                    res_tmp.err().unwrap().into()
                                                }
                                            } else {
                                                res1.err().unwrap().into()
                                            }
                                        }
                                    };
                                    let res_tmp = ParseResult::unify_sequence(ok, res2);
                                    if let ParseResult::Ok(ok) = res_tmp {
                                        ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                    } else {
                                        res_tmp.err().unwrap().into()
                                    }
                                } else {
                                    res1.err().unwrap().into()
                                }
                            }
                        };
                        if let ParseResult::Ok(ok) = res {
                            ok.map(|(e0, e1, e2)| (|e0, e1, e2| e0 * e2)(e0, e1, e2))
                                .into()
                        } else {
                            res.err().unwrap().into()
                        }
                    }
                };
                let res2 = {
                    {
                        let res1 = {
                            {
                                let res = {
                                    {
                                        let res1 = { self.parse_mul_expr(src.clone(), idx) };
                                        if let ParseResult::Ok(ok) = res1 {
                                            let src = ok.furthest_it.clone();
                                            let idx = ok.matched;
                                            let res2 = {
                                                {
                                                    let res1 = {
                                                        {
                                                            let mut src2 = src.clone();
                                                            if let Some(v) = src2.next() {
                                                                if Self::matches(&v, &TokTy::Div) {
                                                                    ParseOk {
                                                                        matched: idx + 1,
                                                                        furthest_it: src2,
                                                                        furthest_error: None,
                                                                        value: (v),
                                                                    }
                                                                    .into()
                                                                } else {
                                                                    let got = Self::show_found(&v);
                                                                    ParseErr::single(
                                                                        idx,
                                                                        got,
                                                                        curr_rule,
                                                                        Self::show_expected(
                                                                            &TokTy::Div,
                                                                        ),
                                                                    )
                                                                    .into()
                                                                }
                                                            } else {
                                                                ParseErr::single(
                                                                    idx,
                                                                    "end of input".into(),
                                                                    curr_rule,
                                                                    Self::show_expected(
                                                                        &TokTy::Div,
                                                                    ),
                                                                )
                                                                .into()
                                                            }
                                                        }
                                                    };
                                                    if let ParseResult::Ok(ok) = res1 {
                                                        let src = ok.furthest_it.clone();
                                                        let idx = ok.matched;
                                                        let res2 =
                                                            { self.parse_atom(src.clone(), idx) };
                                                        let res_tmp =
                                                            ParseResult::unify_sequence(ok, res2);
                                                        if let ParseResult::Ok(ok) = res_tmp {
                                                            ok.map(|((e1), (e2))| (e1, e2)).into()
                                                        } else {
                                                            res_tmp.err().unwrap().into()
                                                        }
                                                    } else {
                                                        res1.err().unwrap().into()
                                                    }
                                                }
                                            };
                                            let res_tmp = ParseResult::unify_sequence(ok, res2);
                                            if let ParseResult::Ok(ok) = res_tmp {
                                                ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                            } else {
                                                res_tmp.err().unwrap().into()
                                            }
                                        } else {
                                            res1.err().unwrap().into()
                                        }
                                    }
                                };
                                if let ParseResult::Ok(ok) = res {
                                    ok.map(|(e0, e1, e2)| (|e0, e1, e2| e0 / e2)(e0, e1, e2))
                                        .into()
                                } else {
                                    res.err().unwrap().into()
                                }
                            }
                        };
                        let res2 = { self.parse_atom(src.clone(), idx) };
                        ParseResult::unify_alternatives(res1, res2)
                    }
                };
                ParseResult::unify_alternatives(res1, res2)
            }
        };
        if tmp_res.is_ok() && old.furthest_look() < tmp_res.furthest_look() {
            let new_old = insert_and_get(
                &mut self.memo_mul_expr,
                idx,
                drec::DirectRec::Recurse(tmp_res),
            )
            .parse_result()
            .clone();
            return self.grow_mul_expr(src, idx, new_old);
        }
        let updated = ParseResult::unify_alternatives(tmp_res, old);
        return insert_and_get(
            &mut self.memo_mul_expr,
            idx,
            drec::DirectRec::Recurse(updated),
        )
        .parse_result()
        .clone();
    }
    pub fn mul_expr(&mut self, src: I) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        self.parse_mul_expr(src, 0)
    }
    fn parse_mul_expr(&mut self, src: I, idx: usize) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        let curr_rule = "mul_expr";
        match self.memo_mul_expr.get(&idx) {
            None => {
                self.memo_mul_expr
                    .insert(idx, drec::DirectRec::Base(ParseErr::new().into()));
                let tmp_res = {
                    {
                        let res1 = {
                            {
                                let res = {
                                    {
                                        let res1 = { self.parse_mul_expr(src.clone(), idx) };
                                        if let ParseResult::Ok(ok) = res1 {
                                            let src = ok.furthest_it.clone();
                                            let idx = ok.matched;
                                            let res2 = {
                                                {
                                                    let res1 = {
                                                        {
                                                            let mut src2 = src.clone();
                                                            if let Some(v) = src2.next() {
                                                                if Self::matches(&v, &TokTy::Mul) {
                                                                    ParseOk {
                                                                        matched: idx + 1,
                                                                        furthest_it: src2,
                                                                        furthest_error: None,
                                                                        value: (v),
                                                                    }
                                                                    .into()
                                                                } else {
                                                                    let got = Self::show_found(&v);
                                                                    ParseErr::single(
                                                                        idx,
                                                                        got,
                                                                        curr_rule,
                                                                        Self::show_expected(
                                                                            &TokTy::Mul,
                                                                        ),
                                                                    )
                                                                    .into()
                                                                }
                                                            } else {
                                                                ParseErr::single(
                                                                    idx,
                                                                    "end of input".into(),
                                                                    curr_rule,
                                                                    Self::show_expected(
                                                                        &TokTy::Mul,
                                                                    ),
                                                                )
                                                                .into()
                                                            }
                                                        }
                                                    };
                                                    if let ParseResult::Ok(ok) = res1 {
                                                        let src = ok.furthest_it.clone();
                                                        let idx = ok.matched;
                                                        let res2 =
                                                            { self.parse_atom(src.clone(), idx) };
                                                        let res_tmp =
                                                            ParseResult::unify_sequence(ok, res2);
                                                        if let ParseResult::Ok(ok) = res_tmp {
                                                            ok.map(|((e1), (e2))| (e1, e2)).into()
                                                        } else {
                                                            res_tmp.err().unwrap().into()
                                                        }
                                                    } else {
                                                        res1.err().unwrap().into()
                                                    }
                                                }
                                            };
                                            let res_tmp = ParseResult::unify_sequence(ok, res2);
                                            if let ParseResult::Ok(ok) = res_tmp {
                                                ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                            } else {
                                                res_tmp.err().unwrap().into()
                                            }
                                        } else {
                                            res1.err().unwrap().into()
                                        }
                                    }
                                };
                                if let ParseResult::Ok(ok) = res {
                                    ok.map(|(e0, e1, e2)| (|e0, e1, e2| e0 * e2)(e0, e1, e2))
                                        .into()
                                } else {
                                    res.err().unwrap().into()
                                }
                            }
                        };
                        let res2 = {
                            {
                                let res1 = {
                                    {
                                        let res = {
                                            {
                                                let res1 =
                                                    { self.parse_mul_expr(src.clone(), idx) };
                                                if let ParseResult::Ok(ok) = res1 {
                                                    let src = ok.furthest_it.clone();
                                                    let idx = ok.matched;
                                                    let res2 = {
                                                        {
                                                            let res1 = {
                                                                {
                                                                    let mut src2 = src.clone();
                                                                    if let Some(v) = src2.next() {
                                                                        if Self::matches(
                                                                            &v,
                                                                            &TokTy::Div,
                                                                        ) {
                                                                            ParseOk {
                                                                                matched: idx + 1,
                                                                                furthest_it: src2,
                                                                                furthest_error:
                                                                                    None,
                                                                                value: (v),
                                                                            }
                                                                            .into()
                                                                        } else {
                                                                            let got =
                                                                                Self::show_found(
                                                                                    &v,
                                                                                );
                                                                            ParseErr::single(
                                                                                idx,
                                                                                got,
                                                                                curr_rule,
                                                                                Self::show_expected(
                                                                                    &TokTy::Div,
                                                                                ),
                                                                            )
                                                                            .into()
                                                                        }
                                                                    } else {
                                                                        ParseErr::single(
                                                                            idx,
                                                                            "end of input".into(),
                                                                            curr_rule,
                                                                            Self::show_expected(
                                                                                &TokTy::Div,
                                                                            ),
                                                                        )
                                                                        .into()
                                                                    }
                                                                }
                                                            };
                                                            if let ParseResult::Ok(ok) = res1 {
                                                                let src = ok.furthest_it.clone();
                                                                let idx = ok.matched;
                                                                let res2 = {
                                                                    self.parse_atom(
                                                                        src.clone(),
                                                                        idx,
                                                                    )
                                                                };
                                                                let res_tmp =
                                                                    ParseResult::unify_sequence(
                                                                        ok, res2,
                                                                    );
                                                                if let ParseResult::Ok(ok) = res_tmp
                                                                {
                                                                    ok.map(|((e1), (e2))| (e1, e2))
                                                                        .into()
                                                                } else {
                                                                    res_tmp.err().unwrap().into()
                                                                }
                                                            } else {
                                                                res1.err().unwrap().into()
                                                            }
                                                        }
                                                    };
                                                    let res_tmp =
                                                        ParseResult::unify_sequence(ok, res2);
                                                    if let ParseResult::Ok(ok) = res_tmp {
                                                        ok.map(|((e0), (e1, e2))| (e0, e1, e2))
                                                            .into()
                                                    } else {
                                                        res_tmp.err().unwrap().into()
                                                    }
                                                } else {
                                                    res1.err().unwrap().into()
                                                }
                                            }
                                        };
                                        if let ParseResult::Ok(ok) = res {
                                            ok.map(|(e0, e1, e2)| {
                                                (|e0, e1, e2| e0 / e2)(e0, e1, e2)
                                            })
                                            .into()
                                        } else {
                                            res.err().unwrap().into()
                                        }
                                    }
                                };
                                let res2 = { self.parse_atom(src.clone(), idx) };
                                ParseResult::unify_alternatives(res1, res2)
                            }
                        };
                        ParseResult::unify_alternatives(res1, res2)
                    }
                };
                match self.memo_mul_expr.get(&idx).unwrap() {
                    drec::DirectRec::Recurse(_) => {
                        let old = insert_and_get(
                            &mut self.memo_mul_expr,
                            idx,
                            drec::DirectRec::Recurse(tmp_res),
                        )
                        .parse_result()
                        .clone();
                        self.grow_mul_expr(src.clone(), idx, old)
                    }
                    drec::DirectRec::Base(_) => {
                        insert_and_get(&mut self.memo_mul_expr, idx, drec::DirectRec::Stub(tmp_res))
                            .parse_result()
                            .clone()
                    }
                    _ => panic!("Unreachable!"),
                }
            }
            Some(drec::DirectRec::Base(res)) => insert_and_get(
                &mut self.memo_mul_expr,
                idx,
                drec::DirectRec::Recurse(ParseErr::new().into()),
            )
            .parse_result()
            .clone(),
            Some(drec::DirectRec::Stub(res)) | Some(drec::DirectRec::Recurse(res)) => res.clone(),
        }
    }
    pub fn atom(&mut self, src: I) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        self.parse_atom(src, 0)
    }
    fn parse_atom(&mut self, src: I, idx: usize) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        let curr_rule = "atom";
        if let Some(res) = self.memo_atom.get(&idx) {
            res.clone()
        } else {
            let res = {
                {
                    let res1 = {
                        {
                            let res = {
                                {
                                    let mut src2 = src.clone();
                                    if let Some(v) = src2.next() {
                                        if Self::matches(&v, &TokTy::IntLit) {
                                            ParseOk {
                                                matched: idx + 1,
                                                furthest_it: src2,
                                                furthest_error: None,
                                                value: (v),
                                            }
                                            .into()
                                        } else {
                                            let got = Self::show_found(&v);
                                            ParseErr::single(
                                                idx,
                                                got,
                                                curr_rule,
                                                Self::show_expected(&TokTy::IntLit),
                                            )
                                            .into()
                                        }
                                    } else {
                                        ParseErr::single(
                                            idx,
                                            "end of input".into(),
                                            curr_rule,
                                            Self::show_expected(&TokTy::IntLit),
                                        )
                                        .into()
                                    }
                                }
                            };
                            if let ParseResult::Ok(ok) = res {
                                ok.map(|(e0)| (|e0| e0.value.parse::<i32>().unwrap())(e0))
                                    .into()
                            } else {
                                res.err().unwrap().into()
                            }
                        }
                    };
                    let res2 = {
                        {
                            let res = {
                                {
                                    let res1 = {
                                        {
                                            let mut src2 = src.clone();
                                            if let Some(v) = src2.next() {
                                                if Self::matches(&v, &TokTy::LeftParen) {
                                                    ParseOk {
                                                        matched: idx + 1,
                                                        furthest_it: src2,
                                                        furthest_error: None,
                                                        value: (v),
                                                    }
                                                    .into()
                                                } else {
                                                    let got = Self::show_found(&v);
                                                    ParseErr::single(
                                                        idx,
                                                        got,
                                                        curr_rule,
                                                        Self::show_expected(&TokTy::LeftParen),
                                                    )
                                                    .into()
                                                }
                                            } else {
                                                ParseErr::single(
                                                    idx,
                                                    "end of input".into(),
                                                    curr_rule,
                                                    Self::show_expected(&TokTy::LeftParen),
                                                )
                                                .into()
                                            }
                                        }
                                    };
                                    if let ParseResult::Ok(ok) = res1 {
                                        let src = ok.furthest_it.clone();
                                        let idx = ok.matched;
                                        let res2 = {
                                            {
                                                let res1 = { self.parse_expr(src.clone(), idx) };
                                                if let ParseResult::Ok(ok) = res1 {
                                                    let src = ok.furthest_it.clone();
                                                    let idx = ok.matched;
                                                    let res2 = {
                                                        {
                                                            let mut src2 = src.clone();
                                                            if let Some(v) = src2.next() {
                                                                if Self::matches(
                                                                    &v,
                                                                    &TokTy::RightParen,
                                                                ) {
                                                                    ParseOk {
                                                                        matched: idx + 1,
                                                                        furthest_it: src2,
                                                                        furthest_error: None,
                                                                        value: (v),
                                                                    }
                                                                    .into()
                                                                } else {
                                                                    let got = Self::show_found(&v);
                                                                    ParseErr::single(
                                                                        idx,
                                                                        got,
                                                                        curr_rule,
                                                                        Self::show_expected(
                                                                            &TokTy::RightParen,
                                                                        ),
                                                                    )
                                                                    .into()
                                                                }
                                                            } else {
                                                                ParseErr::single(
                                                                    idx,
                                                                    "end of input".into(),
                                                                    curr_rule,
                                                                    Self::show_expected(
                                                                        &TokTy::RightParen,
                                                                    ),
                                                                )
                                                                .into()
                                                            }
                                                        }
                                                    };
                                                    let res_tmp =
                                                        ParseResult::unify_sequence(ok, res2);
                                                    if let ParseResult::Ok(ok) = res_tmp {
                                                        ok.map(|((e1), (e2))| (e1, e2)).into()
                                                    } else {
                                                        res_tmp.err().unwrap().into()
                                                    }
                                                } else {
                                                    res1.err().unwrap().into()
                                                }
                                            }
                                        };
                                        let res_tmp = ParseResult::unify_sequence(ok, res2);
                                        if let ParseResult::Ok(ok) = res_tmp {
                                            ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                        } else {
                                            res_tmp.err().unwrap().into()
                                        }
                                    } else {
                                        res1.err().unwrap().into()
                                    }
                                }
                            };
                            if let ParseResult::Ok(ok) = res {
                                ok.map(|(e0, e1, e2)| (|e0, e1, e2| e1)(e0, e1, e2)).into()
                            } else {
                                res.err().unwrap().into()
                            }
                        }
                    };
                    ParseResult::unify_alternatives(res1, res2)
                }
            };
            insert_and_get(&mut self.memo_atom, idx, res).clone()
        }
    }
    fn grow_rel_expr(&mut self, src: I, idx: usize, old: ParseResult<I, i32>) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        let curr_rule = "rel_expr";
        if old.is_err() {
            return old;
        }
        let tmp_res = {
            {
                let res1 = {
                    {
                        let res = {
                            {
                                let res1 = { self.parse_rel_expr(src.clone(), idx) };
                                if let ParseResult::Ok(ok) = res1 {
                                    let src = ok.furthest_it.clone();
                                    let idx = ok.matched;
                                    let res2 = {
                                        {
                                            let res1 = {
                                                {
                                                    let mut src2 = src.clone();
                                                    if let Some(v) = src2.next() {
                                                        if Self::matches(&v, &TokTy::Gr) {
                                                            ParseOk {
                                                                matched: idx + 1,
                                                                furthest_it: src2,
                                                                furthest_error: None,
                                                                value: (v),
                                                            }
                                                            .into()
                                                        } else {
                                                            let got = Self::show_found(&v);
                                                            ParseErr::single(
                                                                idx,
                                                                got,
                                                                curr_rule,
                                                                Self::show_expected(&TokTy::Gr),
                                                            )
                                                            .into()
                                                        }
                                                    } else {
                                                        ParseErr::single(
                                                            idx,
                                                            "end of input".into(),
                                                            curr_rule,
                                                            Self::show_expected(&TokTy::Gr),
                                                        )
                                                        .into()
                                                    }
                                                }
                                            };
                                            if let ParseResult::Ok(ok) = res1 {
                                                let src = ok.furthest_it.clone();
                                                let idx = ok.matched;
                                                let res2 =
                                                    { self.parse_add_expr(src.clone(), idx) };
                                                let res_tmp = ParseResult::unify_sequence(ok, res2);
                                                if let ParseResult::Ok(ok) = res_tmp {
                                                    ok.map(|((e1), (e2))| (e1, e2)).into()
                                                } else {
                                                    res_tmp.err().unwrap().into()
                                                }
                                            } else {
                                                res1.err().unwrap().into()
                                            }
                                        }
                                    };
                                    let res_tmp = ParseResult::unify_sequence(ok, res2);
                                    if let ParseResult::Ok(ok) = res_tmp {
                                        ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                    } else {
                                        res_tmp.err().unwrap().into()
                                    }
                                } else {
                                    res1.err().unwrap().into()
                                }
                            }
                        };
                        if let ParseResult::Ok(ok) = res {
                            ok.map(|(e0, e1, e2)| {
                                (|e0, e1, e2| {
                                    if e0 > e2 {
                                        1
                                    } else {
                                        0
                                    }
                                })(e0, e1, e2)
                            })
                            .into()
                        } else {
                            res.err().unwrap().into()
                        }
                    }
                };
                let res2 = {
                    {
                        let res1 = {
                            {
                                let res = {
                                    {
                                        let res1 = { self.parse_rel_expr(src.clone(), idx) };
                                        if let ParseResult::Ok(ok) = res1 {
                                            let src = ok.furthest_it.clone();
                                            let idx = ok.matched;
                                            let res2 = {
                                                {
                                                    let res1 = {
                                                        {
                                                            let mut src2 = src.clone();
                                                            if let Some(v) = src2.next() {
                                                                if Self::matches(&v, &TokTy::Le) {
                                                                    ParseOk {
                                                                        matched: idx + 1,
                                                                        furthest_it: src2,
                                                                        furthest_error: None,
                                                                        value: (v),
                                                                    }
                                                                    .into()
                                                                } else {
                                                                    let got = Self::show_found(&v);
                                                                    ParseErr::single(
                                                                        idx,
                                                                        got,
                                                                        curr_rule,
                                                                        Self::show_expected(
                                                                            &TokTy::Le,
                                                                        ),
                                                                    )
                                                                    .into()
                                                                }
                                                            } else {
                                                                ParseErr::single(
                                                                    idx,
                                                                    "end of input".into(),
                                                                    curr_rule,
                                                                    Self::show_expected(&TokTy::Le),
                                                                )
                                                                .into()
                                                            }
                                                        }
                                                    };
                                                    if let ParseResult::Ok(ok) = res1 {
                                                        let src = ok.furthest_it.clone();
                                                        let idx = ok.matched;
                                                        let res2 = {
                                                            self.parse_add_expr(src.clone(), idx)
                                                        };
                                                        let res_tmp =
                                                            ParseResult::unify_sequence(ok, res2);
                                                        if let ParseResult::Ok(ok) = res_tmp {
                                                            ok.map(|((e1), (e2))| (e1, e2)).into()
                                                        } else {
                                                            res_tmp.err().unwrap().into()
                                                        }
                                                    } else {
                                                        res1.err().unwrap().into()
                                                    }
                                                }
                                            };
                                            let res_tmp = ParseResult::unify_sequence(ok, res2);
                                            if let ParseResult::Ok(ok) = res_tmp {
                                                ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                            } else {
                                                res_tmp.err().unwrap().into()
                                            }
                                        } else {
                                            res1.err().unwrap().into()
                                        }
                                    }
                                };
                                if let ParseResult::Ok(ok) = res {
                                    ok.map(|(e0, e1, e2)| {
                                        (|e0, e1, e2| {
                                            if e0 < e2 {
                                                1
                                            } else {
                                                0
                                            }
                                        })(e0, e1, e2)
                                    })
                                    .into()
                                } else {
                                    res.err().unwrap().into()
                                }
                            }
                        };
                        let res2 = {
                            {
                                let res1 = {
                                    {
                                        let res = {
                                            {
                                                let res1 =
                                                    { self.parse_rel_expr(src.clone(), idx) };
                                                if let ParseResult::Ok(ok) = res1 {
                                                    let src = ok.furthest_it.clone();
                                                    let idx = ok.matched;
                                                    let res2 = {
                                                        {
                                                            let res1 = {
                                                                {
                                                                    let mut src2 = src.clone();
                                                                    if let Some(v) = src2.next() {
                                                                        if Self::matches(
                                                                            &v,
                                                                            &TokTy::GrEq,
                                                                        ) {
                                                                            ParseOk {
                                                                                matched: idx + 1,
                                                                                furthest_it: src2,
                                                                                furthest_error:
                                                                                    None,
                                                                                value: (v),
                                                                            }
                                                                            .into()
                                                                        } else {
                                                                            let got =
                                                                                Self::show_found(
                                                                                    &v,
                                                                                );
                                                                            ParseErr::single(
                                                                                idx,
                                                                                got,
                                                                                curr_rule,
                                                                                Self::show_expected(
                                                                                    &TokTy::GrEq,
                                                                                ),
                                                                            )
                                                                            .into()
                                                                        }
                                                                    } else {
                                                                        ParseErr::single(
                                                                            idx,
                                                                            "end of input".into(),
                                                                            curr_rule,
                                                                            Self::show_expected(
                                                                                &TokTy::GrEq,
                                                                            ),
                                                                        )
                                                                        .into()
                                                                    }
                                                                }
                                                            };
                                                            if let ParseResult::Ok(ok) = res1 {
                                                                let src = ok.furthest_it.clone();
                                                                let idx = ok.matched;
                                                                let res2 = {
                                                                    self.parse_add_expr(
                                                                        src.clone(),
                                                                        idx,
                                                                    )
                                                                };
                                                                let res_tmp =
                                                                    ParseResult::unify_sequence(
                                                                        ok, res2,
                                                                    );
                                                                if let ParseResult::Ok(ok) = res_tmp
                                                                {
                                                                    ok.map(|((e1), (e2))| (e1, e2))
                                                                        .into()
                                                                } else {
                                                                    res_tmp.err().unwrap().into()
                                                                }
                                                            } else {
                                                                res1.err().unwrap().into()
                                                            }
                                                        }
                                                    };
                                                    let res_tmp =
                                                        ParseResult::unify_sequence(ok, res2);
                                                    if let ParseResult::Ok(ok) = res_tmp {
                                                        ok.map(|((e0), (e1, e2))| (e0, e1, e2))
                                                            .into()
                                                    } else {
                                                        res_tmp.err().unwrap().into()
                                                    }
                                                } else {
                                                    res1.err().unwrap().into()
                                                }
                                            }
                                        };
                                        if let ParseResult::Ok(ok) = res {
                                            ok.map(|(e0, e1, e2)| {
                                                (|e0, e1, e2| {
                                                    if e0 >= e2 {
                                                        1
                                                    } else {
                                                        0
                                                    }
                                                })(
                                                    e0, e1, e2
                                                )
                                            })
                                            .into()
                                        } else {
                                            res.err().unwrap().into()
                                        }
                                    }
                                };
                                let res2 = {
                                    {
                                        let res1 = {
                                            {
                                                let res = {
                                                    {
                                                        let res1 = {
                                                            self.parse_rel_expr(src.clone(), idx)
                                                        };
                                                        if let ParseResult::Ok(ok) = res1 {
                                                            let src = ok.furthest_it.clone();
                                                            let idx = ok.matched;
                                                            let res2 = {
                                                                {
                                                                    let res1 = {
                                                                        {
                                                                            let mut src2 =
                                                                                src.clone();
                                                                            if let Some(v) =
                                                                                src2.next()
                                                                            {
                                                                                if Self::matches(
                                                                                    &v,
                                                                                    &TokTy::LeEq,
                                                                                ) {
                                                                                    ParseOk {
matched : idx + 1 , furthest_it : src2 , furthest_error : None , value : ( v )
} . into (  )
                                                                                } else {
                                                                                    let got = Self :: show_found ( & v ) ;
                                                                                    ParseErr :: single (
idx , got , curr_rule , Self :: show_expected ( & TokTy :: LeEq ) ) . into (
)
                                                                                }
                                                                            } else {
                                                                                ParseErr :: single (
idx , "end of input" . into (  ) , curr_rule , Self :: show_expected (
& TokTy :: LeEq ) ) . into (  )
                                                                            }
                                                                        }
                                                                    };
                                                                    if let ParseResult::Ok(ok) =
                                                                        res1
                                                                    {
                                                                        let src =
                                                                            ok.furthest_it.clone();
                                                                        let idx = ok.matched;
                                                                        let res2 = {
                                                                            self.parse_add_expr(
                                                                                src.clone(),
                                                                                idx,
                                                                            )
                                                                        };
                                                                        let res_tmp =
ParseResult :: unify_sequence ( ok , res2 ) ;
                                                                        if let ParseResult::Ok(ok) =
                                                                            res_tmp
                                                                        {
                                                                            ok.map(
                                                                                |((e1), (e2))| {
                                                                                    (e1, e2)
                                                                                },
                                                                            )
                                                                            .into()
                                                                        } else {
                                                                            res_tmp
                                                                                .err()
                                                                                .unwrap()
                                                                                .into()
                                                                        }
                                                                    } else {
                                                                        res1.err().unwrap().into()
                                                                    }
                                                                }
                                                            };
                                                            let res_tmp =
                                                                ParseResult::unify_sequence(
                                                                    ok, res2,
                                                                );
                                                            if let ParseResult::Ok(ok) = res_tmp {
                                                                ok.map(|((e0), (e1, e2))| {
                                                                    (e0, e1, e2)
                                                                })
                                                                .into()
                                                            } else {
                                                                res_tmp.err().unwrap().into()
                                                            }
                                                        } else {
                                                            res1.err().unwrap().into()
                                                        }
                                                    }
                                                };
                                                if let ParseResult::Ok(ok) = res {
                                                    ok.map(|(e0, e1, e2)| {
                                                        (|e0, e1, e2| {
                                                            if e0 <= e2 {
                                                                1
                                                            } else {
                                                                0
                                                            }
                                                        })(
                                                            e0, e1, e2
                                                        )
                                                    })
                                                    .into()
                                                } else {
                                                    res.err().unwrap().into()
                                                }
                                            }
                                        };
                                        let res2 = { self.parse_add_expr(src.clone(), idx) };
                                        ParseResult::unify_alternatives(res1, res2)
                                    }
                                };
                                ParseResult::unify_alternatives(res1, res2)
                            }
                        };
                        ParseResult::unify_alternatives(res1, res2)
                    }
                };
                ParseResult::unify_alternatives(res1, res2)
            }
        };
        if tmp_res.is_ok() && old.furthest_look() < tmp_res.furthest_look() {
            let new_old = insert_and_get(
                &mut self.memo_rel_expr,
                idx,
                drec::DirectRec::Recurse(tmp_res),
            )
            .parse_result()
            .clone();
            return self.grow_rel_expr(src, idx, new_old);
        }
        let updated = ParseResult::unify_alternatives(tmp_res, old);
        return insert_and_get(
            &mut self.memo_rel_expr,
            idx,
            drec::DirectRec::Recurse(updated),
        )
        .parse_result()
        .clone();
    }
    pub fn rel_expr(&mut self, src: I) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        self.parse_rel_expr(src, 0)
    }
    fn parse_rel_expr(&mut self, src: I, idx: usize) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        let curr_rule = "rel_expr";
        match self.memo_rel_expr.get(&idx) {
            None => {
                self.memo_rel_expr
                    .insert(idx, drec::DirectRec::Base(ParseErr::new().into()));
                let tmp_res = {
                    {
                        let res1 = {
                            {
                                let res = {
                                    {
                                        let res1 = { self.parse_rel_expr(src.clone(), idx) };
                                        if let ParseResult::Ok(ok) = res1 {
                                            let src = ok.furthest_it.clone();
                                            let idx = ok.matched;
                                            let res2 = {
                                                {
                                                    let res1 = {
                                                        {
                                                            let mut src2 = src.clone();
                                                            if let Some(v) = src2.next() {
                                                                if Self::matches(&v, &TokTy::Gr) {
                                                                    ParseOk {
                                                                        matched: idx + 1,
                                                                        furthest_it: src2,
                                                                        furthest_error: None,
                                                                        value: (v),
                                                                    }
                                                                    .into()
                                                                } else {
                                                                    let got = Self::show_found(&v);
                                                                    ParseErr::single(
                                                                        idx,
                                                                        got,
                                                                        curr_rule,
                                                                        Self::show_expected(
                                                                            &TokTy::Gr,
                                                                        ),
                                                                    )
                                                                    .into()
                                                                }
                                                            } else {
                                                                ParseErr::single(
                                                                    idx,
                                                                    "end of input".into(),
                                                                    curr_rule,
                                                                    Self::show_expected(&TokTy::Gr),
                                                                )
                                                                .into()
                                                            }
                                                        }
                                                    };
                                                    if let ParseResult::Ok(ok) = res1 {
                                                        let src = ok.furthest_it.clone();
                                                        let idx = ok.matched;
                                                        let res2 = {
                                                            self.parse_add_expr(src.clone(), idx)
                                                        };
                                                        let res_tmp =
                                                            ParseResult::unify_sequence(ok, res2);
                                                        if let ParseResult::Ok(ok) = res_tmp {
                                                            ok.map(|((e1), (e2))| (e1, e2)).into()
                                                        } else {
                                                            res_tmp.err().unwrap().into()
                                                        }
                                                    } else {
                                                        res1.err().unwrap().into()
                                                    }
                                                }
                                            };
                                            let res_tmp = ParseResult::unify_sequence(ok, res2);
                                            if let ParseResult::Ok(ok) = res_tmp {
                                                ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                            } else {
                                                res_tmp.err().unwrap().into()
                                            }
                                        } else {
                                            res1.err().unwrap().into()
                                        }
                                    }
                                };
                                if let ParseResult::Ok(ok) = res {
                                    ok.map(|(e0, e1, e2)| {
                                        (|e0, e1, e2| {
                                            if e0 > e2 {
                                                1
                                            } else {
                                                0
                                            }
                                        })(e0, e1, e2)
                                    })
                                    .into()
                                } else {
                                    res.err().unwrap().into()
                                }
                            }
                        };
                        let res2 = {
                            {
                                let res1 = {
                                    {
                                        let res = {
                                            {
                                                let res1 =
                                                    { self.parse_rel_expr(src.clone(), idx) };
                                                if let ParseResult::Ok(ok) = res1 {
                                                    let src = ok.furthest_it.clone();
                                                    let idx = ok.matched;
                                                    let res2 = {
                                                        {
                                                            let res1 = {
                                                                {
                                                                    let mut src2 = src.clone();
                                                                    if let Some(v) = src2.next() {
                                                                        if Self::matches(
                                                                            &v,
                                                                            &TokTy::Le,
                                                                        ) {
                                                                            ParseOk {
                                                                                matched: idx + 1,
                                                                                furthest_it: src2,
                                                                                furthest_error:
                                                                                    None,
                                                                                value: (v),
                                                                            }
                                                                            .into()
                                                                        } else {
                                                                            let got =
                                                                                Self::show_found(
                                                                                    &v,
                                                                                );
                                                                            ParseErr::single(
                                                                                idx,
                                                                                got,
                                                                                curr_rule,
                                                                                Self::show_expected(
                                                                                    &TokTy::Le,
                                                                                ),
                                                                            )
                                                                            .into()
                                                                        }
                                                                    } else {
                                                                        ParseErr::single(
                                                                            idx,
                                                                            "end of input".into(),
                                                                            curr_rule,
                                                                            Self::show_expected(
                                                                                &TokTy::Le,
                                                                            ),
                                                                        )
                                                                        .into()
                                                                    }
                                                                }
                                                            };
                                                            if let ParseResult::Ok(ok) = res1 {
                                                                let src = ok.furthest_it.clone();
                                                                let idx = ok.matched;
                                                                let res2 = {
                                                                    self.parse_add_expr(
                                                                        src.clone(),
                                                                        idx,
                                                                    )
                                                                };
                                                                let res_tmp =
                                                                    ParseResult::unify_sequence(
                                                                        ok, res2,
                                                                    );
                                                                if let ParseResult::Ok(ok) = res_tmp
                                                                {
                                                                    ok.map(|((e1), (e2))| (e1, e2))
                                                                        .into()
                                                                } else {
                                                                    res_tmp.err().unwrap().into()
                                                                }
                                                            } else {
                                                                res1.err().unwrap().into()
                                                            }
                                                        }
                                                    };
                                                    let res_tmp =
                                                        ParseResult::unify_sequence(ok, res2);
                                                    if let ParseResult::Ok(ok) = res_tmp {
                                                        ok.map(|((e0), (e1, e2))| (e0, e1, e2))
                                                            .into()
                                                    } else {
                                                        res_tmp.err().unwrap().into()
                                                    }
                                                } else {
                                                    res1.err().unwrap().into()
                                                }
                                            }
                                        };
                                        if let ParseResult::Ok(ok) = res {
                                            ok.map(|(e0, e1, e2)| {
                                                (|e0, e1, e2| {
                                                    if e0 < e2 {
                                                        1
                                                    } else {
                                                        0
                                                    }
                                                })(
                                                    e0, e1, e2
                                                )
                                            })
                                            .into()
                                        } else {
                                            res.err().unwrap().into()
                                        }
                                    }
                                };
                                let res2 = {
                                    {
                                        let res1 = {
                                            {
                                                let res = {
                                                    {
                                                        let res1 = {
                                                            self.parse_rel_expr(src.clone(), idx)
                                                        };
                                                        if let ParseResult::Ok(ok) = res1 {
                                                            let src = ok.furthest_it.clone();
                                                            let idx = ok.matched;
                                                            let res2 = {
                                                                {
                                                                    let res1 = {
                                                                        {
                                                                            let mut src2 =
                                                                                src.clone();
                                                                            if let Some(v) =
                                                                                src2.next()
                                                                            {
                                                                                if Self::matches(
                                                                                    &v,
                                                                                    &TokTy::GrEq,
                                                                                ) {
                                                                                    ParseOk {
matched : idx + 1 , furthest_it : src2 , furthest_error : None , value : ( v )
} . into (  )
                                                                                } else {
                                                                                    let got = Self :: show_found ( & v ) ;
                                                                                    ParseErr :: single (
idx , got , curr_rule , Self :: show_expected ( & TokTy :: GrEq ) ) . into (
)
                                                                                }
                                                                            } else {
                                                                                ParseErr :: single (
idx , "end of input" . into (  ) , curr_rule , Self :: show_expected (
& TokTy :: GrEq ) ) . into (  )
                                                                            }
                                                                        }
                                                                    };
                                                                    if let ParseResult::Ok(ok) =
                                                                        res1
                                                                    {
                                                                        let src =
                                                                            ok.furthest_it.clone();
                                                                        let idx = ok.matched;
                                                                        let res2 = {
                                                                            self.parse_add_expr(
                                                                                src.clone(),
                                                                                idx,
                                                                            )
                                                                        };
                                                                        let res_tmp =
ParseResult :: unify_sequence ( ok , res2 ) ;
                                                                        if let ParseResult::Ok(ok) =
                                                                            res_tmp
                                                                        {
                                                                            ok.map(
                                                                                |((e1), (e2))| {
                                                                                    (e1, e2)
                                                                                },
                                                                            )
                                                                            .into()
                                                                        } else {
                                                                            res_tmp
                                                                                .err()
                                                                                .unwrap()
                                                                                .into()
                                                                        }
                                                                    } else {
                                                                        res1.err().unwrap().into()
                                                                    }
                                                                }
                                                            };
                                                            let res_tmp =
                                                                ParseResult::unify_sequence(
                                                                    ok, res2,
                                                                );
                                                            if let ParseResult::Ok(ok) = res_tmp {
                                                                ok.map(|((e0), (e1, e2))| {
                                                                    (e0, e1, e2)
                                                                })
                                                                .into()
                                                            } else {
                                                                res_tmp.err().unwrap().into()
                                                            }
                                                        } else {
                                                            res1.err().unwrap().into()
                                                        }
                                                    }
                                                };
                                                if let ParseResult::Ok(ok) = res {
                                                    ok.map(|(e0, e1, e2)| {
                                                        (|e0, e1, e2| {
                                                            if e0 >= e2 {
                                                                1
                                                            } else {
                                                                0
                                                            }
                                                        })(
                                                            e0, e1, e2
                                                        )
                                                    })
                                                    .into()
                                                } else {
                                                    res.err().unwrap().into()
                                                }
                                            }
                                        };
                                        let res2 = {
                                            {
                                                let res1 = {
                                                    {
                                                        let res = {
                                                            {
                                                                let res1 = {
                                                                    self.parse_rel_expr(
                                                                        src.clone(),
                                                                        idx,
                                                                    )
                                                                };
                                                                if let ParseResult::Ok(ok) = res1 {
                                                                    let src =
                                                                        ok.furthest_it.clone();
                                                                    let idx = ok.matched;
                                                                    let res2 = {
                                                                        {
                                                                            let res1 = {
                                                                                {
                                                                                    let mut src2 =
                                                                                        src.clone();
                                                                                    if let Some(v) =
                                                                                        src2.next()
                                                                                    {
                                                                                        if Self :: matches ( & v , & TokTy :: LeEq ) {
ParseOk {
matched : idx + 1 , furthest_it : src2 , furthest_error : None , value : ( v )
} . into (  ) } else {
let got = Self :: show_found ( & v ) ; ParseErr :: single (
idx , got , curr_rule , Self :: show_expected ( & TokTy :: LeEq ) ) . into (
) }
                                                                                    } else {
                                                                                        ParseErr :: single (
idx , "end of input" . into (  ) , curr_rule , Self :: show_expected (
& TokTy :: LeEq ) ) . into (  )
                                                                                    }
                                                                                }
                                                                            };
                                                                            if let ParseResult::Ok(
                                                                                ok,
                                                                            ) = res1
                                                                            {
                                                                                let src = ok
                                                                                    .furthest_it
                                                                                    .clone();
                                                                                let idx =
                                                                                    ok.matched;
                                                                                let res2 = {
                                                                                    self . parse_add_expr ( src . clone (  ) , idx )
                                                                                };
                                                                                let res_tmp =
ParseResult :: unify_sequence ( ok , res2 ) ;
                                                                                if let ParseResult :: Ok ( ok )
= res_tmp { ok . map ( | ( ( e1 ) , ( e2 ) ) | ( e1 , e2 ) ) . into (  ) }
else { res_tmp . err (  ) . unwrap (  ) . into (  ) }
                                                                            } else {
                                                                                res1.err()
                                                                                    .unwrap()
                                                                                    .into()
                                                                            }
                                                                        }
                                                                    };
                                                                    let res_tmp =
                                                                        ParseResult::unify_sequence(
                                                                            ok, res2,
                                                                        );
                                                                    if let ParseResult::Ok(ok) =
                                                                        res_tmp
                                                                    {
                                                                        ok.map(
                                                                            |((e0), (e1, e2))| {
                                                                                (e0, e1, e2)
                                                                            },
                                                                        )
                                                                        .into()
                                                                    } else {
                                                                        res_tmp
                                                                            .err()
                                                                            .unwrap()
                                                                            .into()
                                                                    }
                                                                } else {
                                                                    res1.err().unwrap().into()
                                                                }
                                                            }
                                                        };
                                                        if let ParseResult::Ok(ok) = res {
                                                            ok.map(|(e0, e1, e2)| {
                                                                (|e0, e1, e2| {
                                                                    if e0 <= e2 {
                                                                        1
                                                                    } else {
                                                                        0
                                                                    }
                                                                })(
                                                                    e0, e1, e2
                                                                )
                                                            })
                                                            .into()
                                                        } else {
                                                            res.err().unwrap().into()
                                                        }
                                                    }
                                                };
                                                let res2 =
                                                    { self.parse_add_expr(src.clone(), idx) };
                                                ParseResult::unify_alternatives(res1, res2)
                                            }
                                        };
                                        ParseResult::unify_alternatives(res1, res2)
                                    }
                                };
                                ParseResult::unify_alternatives(res1, res2)
                            }
                        };
                        ParseResult::unify_alternatives(res1, res2)
                    }
                };
                match self.memo_rel_expr.get(&idx).unwrap() {
                    drec::DirectRec::Recurse(_) => {
                        let old = insert_and_get(
                            &mut self.memo_rel_expr,
                            idx,
                            drec::DirectRec::Recurse(tmp_res),
                        )
                        .parse_result()
                        .clone();
                        self.grow_rel_expr(src.clone(), idx, old)
                    }
                    drec::DirectRec::Base(_) => {
                        insert_and_get(&mut self.memo_rel_expr, idx, drec::DirectRec::Stub(tmp_res))
                            .parse_result()
                            .clone()
                    }
                    _ => panic!("Unreachable!"),
                }
            }
            Some(drec::DirectRec::Base(res)) => insert_and_get(
                &mut self.memo_rel_expr,
                idx,
                drec::DirectRec::Recurse(ParseErr::new().into()),
            )
            .parse_result()
            .clone(),
            Some(drec::DirectRec::Stub(res)) | Some(drec::DirectRec::Recurse(res)) => res.clone(),
        }
    }
    fn grow_eqty_expr(
        &mut self,
        src: I,
        idx: usize,
        old: ParseResult<I, i32>,
    ) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        let curr_rule = "eqty_expr";
        if old.is_err() {
            return old;
        }
        let tmp_res = {
            {
                let res1 = {
                    {
                        let res = {
                            {
                                let res1 = { self.parse_eqty_expr(src.clone(), idx) };
                                if let ParseResult::Ok(ok) = res1 {
                                    let src = ok.furthest_it.clone();
                                    let idx = ok.matched;
                                    let res2 = {
                                        {
                                            let res1 = {
                                                {
                                                    let mut src2 = src.clone();
                                                    if let Some(v) = src2.next() {
                                                        if Self::matches(&v, &TokTy::Eq) {
                                                            ParseOk {
                                                                matched: idx + 1,
                                                                furthest_it: src2,
                                                                furthest_error: None,
                                                                value: (v),
                                                            }
                                                            .into()
                                                        } else {
                                                            let got = Self::show_found(&v);
                                                            ParseErr::single(
                                                                idx,
                                                                got,
                                                                curr_rule,
                                                                Self::show_expected(&TokTy::Eq),
                                                            )
                                                            .into()
                                                        }
                                                    } else {
                                                        ParseErr::single(
                                                            idx,
                                                            "end of input".into(),
                                                            curr_rule,
                                                            Self::show_expected(&TokTy::Eq),
                                                        )
                                                        .into()
                                                    }
                                                }
                                            };
                                            if let ParseResult::Ok(ok) = res1 {
                                                let src = ok.furthest_it.clone();
                                                let idx = ok.matched;
                                                let res2 =
                                                    { self.parse_rel_expr(src.clone(), idx) };
                                                let res_tmp = ParseResult::unify_sequence(ok, res2);
                                                if let ParseResult::Ok(ok) = res_tmp {
                                                    ok.map(|((e1), (e2))| (e1, e2)).into()
                                                } else {
                                                    res_tmp.err().unwrap().into()
                                                }
                                            } else {
                                                res1.err().unwrap().into()
                                            }
                                        }
                                    };
                                    let res_tmp = ParseResult::unify_sequence(ok, res2);
                                    if let ParseResult::Ok(ok) = res_tmp {
                                        ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                    } else {
                                        res_tmp.err().unwrap().into()
                                    }
                                } else {
                                    res1.err().unwrap().into()
                                }
                            }
                        };
                        if let ParseResult::Ok(ok) = res {
                            ok.map(|(e0, e1, e2)| {
                                (|e0, e1, e2| {
                                    if e0 == e2 {
                                        1
                                    } else {
                                        0
                                    }
                                })(e0, e1, e2)
                            })
                            .into()
                        } else {
                            res.err().unwrap().into()
                        }
                    }
                };
                let res2 = {
                    {
                        let res1 = {
                            {
                                let res = {
                                    {
                                        let res1 = { self.parse_eqty_expr(src.clone(), idx) };
                                        if let ParseResult::Ok(ok) = res1 {
                                            let src = ok.furthest_it.clone();
                                            let idx = ok.matched;
                                            let res2 = {
                                                {
                                                    let res1 = {
                                                        {
                                                            let mut src2 = src.clone();
                                                            if let Some(v) = src2.next() {
                                                                if Self::matches(&v, &TokTy::Neq) {
                                                                    ParseOk {
                                                                        matched: idx + 1,
                                                                        furthest_it: src2,
                                                                        furthest_error: None,
                                                                        value: (v),
                                                                    }
                                                                    .into()
                                                                } else {
                                                                    let got = Self::show_found(&v);
                                                                    ParseErr::single(
                                                                        idx,
                                                                        got,
                                                                        curr_rule,
                                                                        Self::show_expected(
                                                                            &TokTy::Neq,
                                                                        ),
                                                                    )
                                                                    .into()
                                                                }
                                                            } else {
                                                                ParseErr::single(
                                                                    idx,
                                                                    "end of input".into(),
                                                                    curr_rule,
                                                                    Self::show_expected(
                                                                        &TokTy::Neq,
                                                                    ),
                                                                )
                                                                .into()
                                                            }
                                                        }
                                                    };
                                                    if let ParseResult::Ok(ok) = res1 {
                                                        let src = ok.furthest_it.clone();
                                                        let idx = ok.matched;
                                                        let res2 = {
                                                            self.parse_rel_expr(src.clone(), idx)
                                                        };
                                                        let res_tmp =
                                                            ParseResult::unify_sequence(ok, res2);
                                                        if let ParseResult::Ok(ok) = res_tmp {
                                                            ok.map(|((e1), (e2))| (e1, e2)).into()
                                                        } else {
                                                            res_tmp.err().unwrap().into()
                                                        }
                                                    } else {
                                                        res1.err().unwrap().into()
                                                    }
                                                }
                                            };
                                            let res_tmp = ParseResult::unify_sequence(ok, res2);
                                            if let ParseResult::Ok(ok) = res_tmp {
                                                ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                            } else {
                                                res_tmp.err().unwrap().into()
                                            }
                                        } else {
                                            res1.err().unwrap().into()
                                        }
                                    }
                                };
                                if let ParseResult::Ok(ok) = res {
                                    ok.map(|(e0, e1, e2)| {
                                        (|e0, e1, e2| {
                                            if e0 != e2 {
                                                1
                                            } else {
                                                0
                                            }
                                        })(e0, e1, e2)
                                    })
                                    .into()
                                } else {
                                    res.err().unwrap().into()
                                }
                            }
                        };
                        let res2 = { self.parse_rel_expr(src.clone(), idx) };
                        ParseResult::unify_alternatives(res1, res2)
                    }
                };
                ParseResult::unify_alternatives(res1, res2)
            }
        };
        if tmp_res.is_ok() && old.furthest_look() < tmp_res.furthest_look() {
            let new_old = insert_and_get(
                &mut self.memo_eqty_expr,
                idx,
                drec::DirectRec::Recurse(tmp_res),
            )
            .parse_result()
            .clone();
            return self.grow_eqty_expr(src, idx, new_old);
        }
        let updated = ParseResult::unify_alternatives(tmp_res, old);
        return insert_and_get(
            &mut self.memo_eqty_expr,
            idx,
            drec::DirectRec::Recurse(updated),
        )
        .parse_result()
        .clone();
    }
    pub fn eqty_expr(&mut self, src: I) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        self.parse_eqty_expr(src, 0)
    }
    fn parse_eqty_expr(&mut self, src: I, idx: usize) -> ParseResult<I, i32>
    where
        I: Iterator + Clone,
        I: 'static,
        i32: 'static,
    {
        let curr_rule = "eqty_expr";
        match self.memo_eqty_expr.get(&idx) {
            None => {
                self.memo_eqty_expr
                    .insert(idx, drec::DirectRec::Base(ParseErr::new().into()));
                let tmp_res = {
                    {
                        let res1 = {
                            {
                                let res = {
                                    {
                                        let res1 = { self.parse_eqty_expr(src.clone(), idx) };
                                        if let ParseResult::Ok(ok) = res1 {
                                            let src = ok.furthest_it.clone();
                                            let idx = ok.matched;
                                            let res2 = {
                                                {
                                                    let res1 = {
                                                        {
                                                            let mut src2 = src.clone();
                                                            if let Some(v) = src2.next() {
                                                                if Self::matches(&v, &TokTy::Eq) {
                                                                    ParseOk {
                                                                        matched: idx + 1,
                                                                        furthest_it: src2,
                                                                        furthest_error: None,
                                                                        value: (v),
                                                                    }
                                                                    .into()
                                                                } else {
                                                                    let got = Self::show_found(&v);
                                                                    ParseErr::single(
                                                                        idx,
                                                                        got,
                                                                        curr_rule,
                                                                        Self::show_expected(
                                                                            &TokTy::Eq,
                                                                        ),
                                                                    )
                                                                    .into()
                                                                }
                                                            } else {
                                                                ParseErr::single(
                                                                    idx,
                                                                    "end of input".into(),
                                                                    curr_rule,
                                                                    Self::show_expected(&TokTy::Eq),
                                                                )
                                                                .into()
                                                            }
                                                        }
                                                    };
                                                    if let ParseResult::Ok(ok) = res1 {
                                                        let src = ok.furthest_it.clone();
                                                        let idx = ok.matched;
                                                        let res2 = {
                                                            self.parse_rel_expr(src.clone(), idx)
                                                        };
                                                        let res_tmp =
                                                            ParseResult::unify_sequence(ok, res2);
                                                        if let ParseResult::Ok(ok) = res_tmp {
                                                            ok.map(|((e1), (e2))| (e1, e2)).into()
                                                        } else {
                                                            res_tmp.err().unwrap().into()
                                                        }
                                                    } else {
                                                        res1.err().unwrap().into()
                                                    }
                                                }
                                            };
                                            let res_tmp = ParseResult::unify_sequence(ok, res2);
                                            if let ParseResult::Ok(ok) = res_tmp {
                                                ok.map(|((e0), (e1, e2))| (e0, e1, e2)).into()
                                            } else {
                                                res_tmp.err().unwrap().into()
                                            }
                                        } else {
                                            res1.err().unwrap().into()
                                        }
                                    }
                                };
                                if let ParseResult::Ok(ok) = res {
                                    ok.map(|(e0, e1, e2)| {
                                        (|e0, e1, e2| {
                                            if e0 == e2 {
                                                1
                                            } else {
                                                0
                                            }
                                        })(e0, e1, e2)
                                    })
                                    .into()
                                } else {
                                    res.err().unwrap().into()
                                }
                            }
                        };
                        let res2 = {
                            {
                                let res1 = {
                                    {
                                        let res = {
                                            {
                                                let res1 =
                                                    { self.parse_eqty_expr(src.clone(), idx) };
                                                if let ParseResult::Ok(ok) = res1 {
                                                    let src = ok.furthest_it.clone();
                                                    let idx = ok.matched;
                                                    let res2 = {
                                                        {
                                                            let res1 = {
                                                                {
                                                                    let mut src2 = src.clone();
                                                                    if let Some(v) = src2.next() {
                                                                        if Self::matches(
                                                                            &v,
                                                                            &TokTy::Neq,
                                                                        ) {
                                                                            ParseOk {
                                                                                matched: idx + 1,
                                                                                furthest_it: src2,
                                                                                furthest_error:
                                                                                    None,
                                                                                value: (v),
                                                                            }
                                                                            .into()
                                                                        } else {
                                                                            let got =
                                                                                Self::show_found(
                                                                                    &v,
                                                                                );
                                                                            ParseErr::single(
                                                                                idx,
                                                                                got,
                                                                                curr_rule,
                                                                                Self::show_expected(
                                                                                    &TokTy::Neq,
                                                                                ),
                                                                            )
                                                                            .into()
                                                                        }
                                                                    } else {
                                                                        ParseErr::single(
                                                                            idx,
                                                                            "end of input".into(),
                                                                            curr_rule,
                                                                            Self::show_expected(
                                                                                &TokTy::Neq,
                                                                            ),
                                                                        )
                                                                        .into()
                                                                    }
                                                                }
                                                            };
                                                            if let ParseResult::Ok(ok) = res1 {
                                                                let src = ok.furthest_it.clone();
                                                                let idx = ok.matched;
                                                                let res2 = {
                                                                    self.parse_rel_expr(
                                                                        src.clone(),
                                                                        idx,
                                                                    )
                                                                };
                                                                let res_tmp =
                                                                    ParseResult::unify_sequence(
                                                                        ok, res2,
                                                                    );
                                                                if let ParseResult::Ok(ok) = res_tmp
                                                                {
                                                                    ok.map(|((e1), (e2))| (e1, e2))
                                                                        .into()
                                                                } else {
                                                                    res_tmp.err().unwrap().into()
                                                                }
                                                            } else {
                                                                res1.err().unwrap().into()
                                                            }
                                                        }
                                                    };
                                                    let res_tmp =
                                                        ParseResult::unify_sequence(ok, res2);
                                                    if let ParseResult::Ok(ok) = res_tmp {
                                                        ok.map(|((e0), (e1, e2))| (e0, e1, e2))
                                                            .into()
                                                    } else {
                                                        res_tmp.err().unwrap().into()
                                                    }
                                                } else {
                                                    res1.err().unwrap().into()
                                                }
                                            }
                                        };
                                        if let ParseResult::Ok(ok) = res {
                                            ok.map(|(e0, e1, e2)| {
                                                (|e0, e1, e2| {
                                                    if e0 != e2 {
                                                        1
                                                    } else {
                                                        0
                                                    }
                                                })(
                                                    e0, e1, e2
                                                )
                                            })
                                            .into()
                                        } else {
                                            res.err().unwrap().into()
                                        }
                                    }
                                };
                                let res2 = { self.parse_rel_expr(src.clone(), idx) };
                                ParseResult::unify_alternatives(res1, res2)
                            }
                        };
                        ParseResult::unify_alternatives(res1, res2)
                    }
                };
                match self.memo_eqty_expr.get(&idx).unwrap() {
                    drec::DirectRec::Recurse(_) => {
                        let old = insert_and_get(
                            &mut self.memo_eqty_expr,
                            idx,
                            drec::DirectRec::Recurse(tmp_res),
                        )
                        .parse_result()
                        .clone();
                        self.grow_eqty_expr(src.clone(), idx, old)
                    }
                    drec::DirectRec::Base(_) => insert_and_get(
                        &mut self.memo_eqty_expr,
                        idx,
                        drec::DirectRec::Stub(tmp_res),
                    )
                    .parse_result()
                    .clone(),
                    _ => panic!("Unreachable!"),
                }
            }
            Some(drec::DirectRec::Base(res)) => insert_and_get(
                &mut self.memo_eqty_expr,
                idx,
                drec::DirectRec::Recurse(ParseErr::new().into()),
            )
            .parse_result()
            .clone(),
            Some(drec::DirectRec::Stub(res)) | Some(drec::DirectRec::Recurse(res)) => res.clone(),
        }
    }
}
fn insert_and_get<K, V>(m: &mut HashMap<K, V>, k: K, v: V) -> &V
where
    K: Clone + Eq + Hash,
{
    m.insert(k.clone(), v);
    m.get(&k).unwrap()
}


    impl <I> Match<TokTy> for Parser<I> where I : Iterator<Item = Token<TokTy>> {
        fn matches(a: &Token<TokTy>, b: &TokTy) -> bool {
            a.kind == *b
        }
    }

    impl <I> ShowExpected<TokTy> for Parser<I> where I : Iterator<Item = Token<TokTy>> {
        fn show_expected(t: &TokTy) -> String {
            "<TokTy>".into()
        }
    }

    impl <I> ShowFound for Parser<I> where I : Iterator<Item = Token<TokTy>> {
        fn show_found(t: &Token<TokTy>) -> String {
            t.value.clone()
        }
    }
}

/*
impl <I> Match<char> for MyParser<I> where I : Iterator<Item = char> {
    fn matches(a: &char, b: &char) -> bool {
        *a == *b
    }
}
*/

fn main() {
    let mut lexer = TokTy::lexer();
    let mut tokens = Vec::new();

    let m = lexer.modify(&tokens, 0..0, "1+2+3");
    tokens.splice(m.erased, m.inserted);

    let mut parser = peg::Parser::new();
    let r = parser.expr(tokens.iter().cloned());
    if r.is_ok() {
        let ok = r.ok().unwrap();
        let val = ok.value;
        let mlen = ok.matched;
        println!("Ok: {:?} (matched: {})", val, mlen);
    }
    else {
        let err = r.err().unwrap();
        println!("Err:");
        for (rule, element) in err.elements {
            print!("  While parsing {} expected: ", rule);

            let mut fst = true;
            for tok in element.expected_elements {
                if !fst {
                    print!(" or ");
                }
                fst = false;
                print!("{}", tok);
            }
            println!();
        }
        println!("But got '{}'", err.found_element);
    }

    /*
    // Creating a lexer
    let mut lexer = MyTokenType::lexer();
    let mut tokens = Vec::new();
    // Modify
    let m = lexer.modify(&tokens, 0..0, "hello world");
    tokens.splice(m.erased, m.inserted);
    print_tokens(lexer.source(), &tokens);
    // Modify
    let m = lexer.modify(&tokens, 5..5, " there");
    tokens.splice(m.erased, m.inserted);
    print_tokens(lexer.source(), &tokens);
    */
}
