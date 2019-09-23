
extern crate yk_lexer;

use yk_lexer::{TokenType, Lexer};

#[derive(yk_lexer::Lexer, PartialEq, Eq, Debug)]
enum MyTokenType {
    #[error]
    Error,

    #[end]
    End,

    #[regex("[c_ident]")]
    Ident,

    #[token("foo")]
    Bar,
}

fn main() {
    let mut lexer = MyTokenType::with_source("foo world");
    loop {
        let tok = lexer.next_token();
        println!("{} - {:?}", tok.value, tok.kind);
        if tok.kind == MyTokenType::Error || tok.kind == MyTokenType::End {
            break;
        }
    }
}
