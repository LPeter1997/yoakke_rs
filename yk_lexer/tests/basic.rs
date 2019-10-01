
mod common;

use yk_lexer::{Position, Token, TokenType, Lexer};
use common::assert_iter_eq;

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

#[test]
fn lex_empty() {
    let mut lexer = TokenKind::lexer();
    assert_iter_eq([
        Token{
            range: 0..0,
            kind: TokenKind::End,
            position: Position::new(),
            lookahead: 0,
        }
    ].iter().cloned(), lexer.iter());
}
