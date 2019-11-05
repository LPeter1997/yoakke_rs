extern crate yk_lexer;
extern crate yk_parser;

use std::io::{self, BufRead, Read, Bytes};
use std::time::{Duration, Instant};
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
            | "(" add_expr ")" { $1 }
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

fn parse_number(i: &mut Bytes<impl Read>) -> usize {
    let mut n: usize = 0;
    let mut has_parsed = false;
    while let Some(res) = i.next() {
        has_parsed = true;
        let dig = res.unwrap();
        if dig == ';' as u8 {
            break;
        }
        n = n * 10 + ((dig - ('0' as u8)) as usize);
    }
    if !has_parsed {
        std::process::exit(0);
    }
    n
}

fn parse_content(i: &mut Bytes<impl Read>, len: usize) -> String {
    let mut v = Vec::new();
    for _ in 0..len {
        if let Some(res) = i.next() {
            let c = res.unwrap();
            v.push(c);
        }
        else {
            break;
        }
    }
    String::from_utf8(v).unwrap()
}

fn main() {
    let mut nonincr_source = String::new();

    let mut incr_tokens = Vec::new();
    let mut incr_lexer = Tok::lexer();
    let mut incr_parser = expr::Parser::new();

    let stdin = io::stdin();
    let stdin_lock = stdin.lock();
    let mut bytes = stdin_lock.bytes();
    loop {
        let offset = parse_number(&mut bytes);
        let removed = parse_number(&mut bytes);
        let inserted = parse_number(&mut bytes);
        let inserted_content = parse_content(&mut bytes, inserted);

        // Skip newline
        bytes.next();
        bytes.next();

        let removed_range = offset..(offset + removed);

        // Nonincremental
        let nir = {
            let mut lexer = Tok::lexer();
            let mut tokens = Vec::new();
            let mut parser = expr::Parser::new();

            let start = Instant::now();

            nonincr_source.drain(removed_range.clone());
            nonincr_source.insert_str(offset, &inserted_content);

            let m = lexer.modify(&tokens, 0..0, &nonincr_source);
            tokens.splice(m.erased, m.inserted);

            let r = parser.expr(tokens.iter().cloned());

            let elapsed = start.elapsed();
            println!("Full-parse took: {}", elapsed.as_millis());

            res_to_str(r)
        };

        let ir = {
            let start = Instant::now();

            let m = incr_lexer.modify(&incr_tokens, removed_range, &inserted_content);
            let m = m.apply(&mut incr_tokens);

            incr_parser.invalidate(m.erased, m.inserted);

            let r = incr_parser.expr(incr_tokens.iter().cloned());

            let elapsed = start.elapsed();
            println!("Incremental-parse took: {}", elapsed.as_millis());

            res_to_str(r)
        };

        println!("{} == {}", nir, ir);
    }
}
