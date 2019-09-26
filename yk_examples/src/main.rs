
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

    #[regex(r"[ \r\n]")]
    #[ignore]
    Whitespace,
}

fn main() {
    // Creating a lexer
    let mut lexer = MyTokenType::lexer();
    // Modify, re-lex the whole thing again
    lexer.modify(&[], 0..0, "hello fo foo_ foo world
haha this is a new    line
mmlul
foo foo foo_");
    // Iterating over all tokens
    for tok in lexer.iter() {
        println!("{:?} - {:?} [{:?}] - ({} - {})", &lexer.source()[tok.range.clone()], tok.kind, tok.position, tok.range.end, tok.lookahead);
    }
    /*
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
    */
}
