
extern crate yk_lexer;

use yk_lexer::{TokenType, Lexer};

#[derive(yk_lexer::Lexer, PartialEq, Eq, Debug)]
enum MyTokenType {
    #[error]
    Error,

    #[end]
    End,

    #[c_ident]
    Ident,

    #[token("foo")]
    Bar,

    #[regex(" ")]
    #[ignore]
    Whitespace,
}

fn main() {
    let mut lexer = MyTokenType::with_source("foo_ world");
    loop {
        let tok = lexer.next_token();
        println!("'{}' - {:?}", tok.value, tok.kind);
        if tok.kind == MyTokenType::End {
            break;
        }
    }
}
