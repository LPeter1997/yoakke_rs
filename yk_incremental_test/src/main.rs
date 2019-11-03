extern crate yk_lexer;
extern crate yk_parser;

use std::io::{self, BufRead};
use yk_lexer::{Lexer, TokenType, Token};
use yk_parser::ParseResult;

#[derive(Lexer, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tok {
    #[error] Error,
    #[end] End,

    #[regex(r"[ \r\n\t]")] #[ignore] Ws,

    #[regex(r"[0-9]+")] IntLit,

    #[token("+")] Add,
    #[token("-")] Sub,
    #[token("*")] Mul,
    #[token("/")] Div,
    #[token("^")] Exp,

    #[token("(")] LeftParen,
    #[token(")")] RightParen,
}

mod expr {
    use std::convert::TryInto;
    use crate::Tok;
    use yk_lexer::Token;
    use yk_parser::yk_parser;

    yk_parser!{
        item = Token<Tok>;

        type = i32;

        expr ::= add_expr $end { $0 };

        add_expr ::=
            | add_expr "+" mul_expr { $0 + $2 }
            | add_expr "-" mul_expr { $0 - $2 }
            | mul_expr
            ;

        mul_expr ::=
            | mul_expr "*" exp_expr { $0 * $2 }
            | mul_expr "/" exp_expr { $0 / $2 }
            | exp_expr
            ;

        exp_expr ::=
            | atom "^" exp_expr { i32::pow($0, i32::try_into($2).unwrap()) }
            | atom
            ;

        atom ::=
            | Tok::IntLit { $0.value.parse::<i32>().unwrap() }
            | "(" expr ")" { $1 }
            ;
    }

    impl Match<EndOfInput> for Parser {
        fn matches(a: &Token<Tok>, b: &EndOfInput) -> bool {
            a.kind == Tok::End
        }

        fn show_expected(t: &EndOfInput) -> String {
            "end of input".into()
        }
    }

    impl Match<Tok> for Parser {
        fn matches(a: &Token<Tok>, b: &Tok) -> bool {
            a.kind == *b
        }

        fn show_expected(t: &Tok) -> String {
            format!("{:?}", t)
        }
    }

    impl Match<&str> for Parser {
        fn matches(a: &Token<Tok>, b: &&str) -> bool {
            a.value == *b
        }

        fn show_expected(t: &&str) -> String {
            (*t).into()
        }
    }
}

fn res_to_str(res: ParseResult<i32, Token<Tok>>) -> String {
    if res.is_err() {
        "err".into()
    }
    else {
        let val = res.ok().unwrap().value;
        format!("{}", val)
    }
}

fn main() {
    let mut nonincr_source = String::new();

    let mut incr_tokens = Vec::new();
    let mut incr_lexer = Tok::lexer();
    let mut incr_parser = expr::Parser::new();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut pieces = line.split(';');

        let offset = pieces.next().unwrap().parse::<usize>().unwrap();
        let removed = pieces.next().unwrap().parse::<usize>().unwrap();
        let inserted = pieces.next().unwrap().parse::<usize>().unwrap();
        let inserted_content = pieces.next().unwrap();

        let removed_range = offset..(offset + removed);

        // Nonincremental
        let nir = {
            nonincr_source.drain(removed_range.clone());
            nonincr_source.insert_str(offset, inserted_content);

            let mut lexer = Tok::lexer();
            let mut tokens = Vec::new();

            let m = lexer.modify(&tokens, 0..0, &nonincr_source);
            tokens.splice(m.erased, m.inserted);

            let mut parser = expr::Parser::new();
            let r = parser.expr(tokens.iter().cloned());

            res_to_str(r)
        };

        let ir = {
            let m = incr_lexer.modify(&incr_tokens, removed_range, &inserted_content);

            // TODO: Move this to modify()?
            // We need to shift the existing tokens
            for t in &mut incr_tokens[m.erased.end..] {
                t.shift(m.offset);
            }
            let ilen = m.inserted.len();
            incr_tokens.splice(m.erased.clone(), m.inserted);
            // TODO end ///////////////////

            incr_parser.invalidate(m.erased, ilen);

            let r = incr_parser.expr(incr_tokens.iter().cloned());

            res_to_str(r)
        };

        println!("{} == {}", nir, ir);
    }
}
