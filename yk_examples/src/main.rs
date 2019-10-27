
extern crate yk_lexer;
extern crate yk_parser;

use std::io::{self, BufRead};
use yk_lexer::{Token, TokenType, Lexer};
use yk_parser::{yk_parser, ParseResult, Match};

#[derive(Lexer, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokTy {
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

    fn btoi(b: bool) -> i32 { if b { 1 } else { 0 } }

    yk_parser!{
        item: Token<TokTy>;
        type: i32;

        expr ::= eq_expr;

        eq_expr ::=
            | eq_expr "==" rel_expr { btoi(e0 == e2) }
            | eq_expr "!=" rel_expr { btoi(e0 != e2) }
            | rel_expr
            ;

        rel_expr ::=
            | rel_expr ">" add_expr { btoi(e0 > e2) }
            | rel_expr "<" add_expr { btoi(e0 < e2) }
            | rel_expr ">=" add_expr { btoi(e0 >= e2) }
            | rel_expr "<=" add_expr { btoi(e0 <= e2) }
            | add_expr
            ;

        add_expr ::=
            | add_expr "+" mul_expr { e0 + e2 }
            | add_expr "-" mul_expr { e0 - e2 }
            | mul_expr
            ;

        mul_expr ::=
            | mul_expr "*" atom { e0 * e2 }
            | mul_expr "/" atom { e0 / e2 }
            | atom
            ;

        atom ::=
            | TokTy::IntLit { e0.value.parse::<i32>().unwrap() }
            | "(" expr ")" { e1 }
            ;
    }

    impl Match<TokTy> for Parser {
        fn matches(a: &Token<TokTy>, b: &TokTy) -> bool {
            a.kind == *b
        }

        fn show_expected(t: &TokTy) -> String {
            "<TokTy>".into()
        }
    }

    impl Match<&str> for Parser {
        fn matches(a: &Token<TokTy>, b: &&str) -> bool {
            a.value == *b
        }

        fn show_expected(t: &&str) -> String {
            (*t).into()
        }
    }

    impl ShowFound for Parser {
        fn show_found(t: &Token<TokTy>) -> String {
            t.value.clone()
        }
    }
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {

    let mut lexer = TokTy::lexer();
    let mut tokens = Vec::new();

    let m = lexer.modify(&tokens, 0..0, &line.unwrap());
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

    }
}
