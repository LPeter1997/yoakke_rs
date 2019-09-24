
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
    // Creating a lexer
    let mut lexer = MyTokenType::lexer();
    // Iterating over all tokens
    for tok in &lexer {
        println!("'{}' - {:?}", tok.value, tok.kind);
    }
    // Modification
    // Erased range, inserted text
    for m in lexer.source_modification(10..20, "Hello there") {
        match m {
            Modification::EraseToken(index) => { /* ... */ },
            Modification::InsertToken(index, token) => { /* ... */ },
        }
    }
}
