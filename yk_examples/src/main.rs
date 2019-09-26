
extern crate yk_lexer;
extern crate yk_parser;

use yk_lexer::{TokenType, Lexer};
use yk_parser::yk_parser;

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

    #[regex(r"[ \r\n]")]
    #[ignore]
    Whitespace,
}

fn print_tokens<T>(src: &str, tokens: &[yk_lexer::Token<T>]) where T : std::fmt::Debug {
    for t in tokens {
        println!("{:?} - {:?} [{:?}]", &src[t.range.clone()], t.kind, t.position);
    }
}

yk_parser!{
    foo ::=
          | 1 { }
          | 2 {}
          ;

    bar ::= 1 {};
}

fn main() {
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
