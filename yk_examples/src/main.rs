
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

yk_parser!{
    expr ::=
        | addition
        ;

    addition ::=
        | addition '+' multiplication { e0 + e1 }
        | addition '-' multiplication { e0 - e1 }
        ;

    multiplication ::=
        | multiplication '*' exponentiation { e0 * e1 }
        | multiplication '/' exponentiation { e0 / e1 }
        ;

    exponentiation ::=
        | atomic '^' exponentiation { i32::pow(e0, e1) }
        ;

    atomic ::=
        | IntLit { to_i32(e0) }
        | '(' expr ')' { e1 }
        ;
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
