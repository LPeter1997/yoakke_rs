
extern crate yk_lexer;
extern crate rand;

mod rnd;
mod str_gen;
mod fuzz_gen;

use yk_lexer::{StandardLexer, Lexer, TokenType, Token};
use str_gen::*;
use fuzz_gen::*;

#[derive(Lexer, Clone, PartialEq, Eq, Debug)]
enum TokenKind {
    #[error]
    Error,

    #[end]
    End,

    #[regex(r"[ \t\r\n]")]
    #[ignore]
    Ws,

    #[c_ident]
    Ident,

    #[regex(r"[0-9]+")]
    IntLit,

    #[token("(")]
    LP,

    #[token(")")]
    RP,

    #[token("if")]
    KwIf,

    #[token("else")]
    KwElse,
}

fn main() {
    let rs = RandomStringGenerator::with_len_and_charset(5..15, "abcdef");
    println!("{}", rs.generate());
    println!("{}", rs.generate());
    println!("{}", rs.generate());
}
