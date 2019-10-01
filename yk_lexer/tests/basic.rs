
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

#[test]
fn lex_spaces() {
    let mut lexer = TokenKind::lexer();
    lexer.modify(&[], 0..0, "  \r\n  \n  \t  \t\r  ");
    assert_iter_eq([
        Token{
            range: 16..16,
            kind: TokenKind::End,
            position: Position{ line: 3, column: 1 },
            lookahead: 0,
        }
    ].iter().cloned(), lexer.iter());
}

#[test]
fn lex_simple() {
    let mut lexer = TokenKind::lexer();
    lexer.modify(&[], 0..0, "if if_ _if 123(else)");
    assert_iter_eq([
        Token{
            range: 0..2,
            kind: TokenKind::KwIf,
            position: Position{ line: 0, column: 0 },
            lookahead: 1,
        },
        Token{
            range: 3..6,
            kind: TokenKind::Ident,
            position: Position{ line: 0, column: 3 },
            lookahead: 1,
        },
        Token{
            range: 7..10,
            kind: TokenKind::Ident,
            position: Position{ line: 0, column: 7 },
            lookahead: 1,
        },
        Token{
            range: 11..14,
            kind: TokenKind::IntLit,
            position: Position{ line: 0, column: 11 },
            lookahead: 1,
        },
        Token{
            range: 14..15,
            kind: TokenKind::LP,
            position: Position{ line: 0, column: 14 },
            lookahead: 1,
        },
        Token{
            range: 15..19,
            kind: TokenKind::KwElse,
            position: Position{ line: 0, column: 15 },
            lookahead: 1,
        },
        Token{
            range: 19..20,
            kind: TokenKind::RP,
            position: Position{ line: 0, column: 19 },
            lookahead: 0,
        },
        Token{
            range: 20..20,
            kind: TokenKind::End,
            position: Position{ line: 0, column: 20 },
            lookahead: 0,
        }
    ].iter().cloned(), lexer.iter());
}
