
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
    // Collecting them all
    let mut toks: Vec<_> = lexer.collect();
    // Modification
    // Erased range, inserted text
    for m in lexer.source_modification(&toks, 10..20, ins_str.len()) {
        match m {
            Modification::EraseToken(index) => { /* ... */ },
            Modification::InsertToken(index, token) => { /* ... */ },
        }
    }
    // Modification in separate batches
    let modif = lexer.source_modification(&toks, 10..20, ins_str.len());
    toks.drain(modif.erased_range);
    for (index, tok) in modif.inserted {
        toks.insert(index, tok);
    }
    // Batch modification
    let modif = lexer.source_modification(&toks, 10..20, ins_str.len());
    toks.drain(modif.erased_range);
    let start = modif.inserted.start;
    let ins: Vec<_> = modif.inserted.map(|(_, t)| r).collect();
    toks.extend(start, ins);
}
