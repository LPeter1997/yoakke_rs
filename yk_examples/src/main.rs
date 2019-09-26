
extern crate yk_lexer;
extern crate yk_parser;

use yk_lexer::{TokenType, Lexer};
use yk_parser::yk_parser;

#[derive(yk_lexer::Lexer, PartialEq, Eq, Debug)]
enum TokTy {
    #[error]
    Error,

    #[end]
    End,

    #[regex(r"[ \r\n]")]
    #[ignore]
    Whitespace,

    #[c_ident]
    Ident,

    #[regex("[0-9]+")]
    IntLit,

    #[token("+")]
    Add,

    #[token("-")]
    Sub,

    #[token("*")]
    Mul,

    #[token("/")]
    Div,

    #[token("^")]
    Exp,

    #[token("(")]
    LP,

    #[token(")")]
    RP,
}

yk_parser!{
    expr ::=
        | addition
        ;

    addition ::=
        | addition TokTy::Add multiplication { e0 + e1 }
        | addition TokTy::Sub multiplication { e0 - e1 }
        ;

    multiplication ::=
        | multiplication TokTy::Mul exponentiation { e0 * e1 }
        | multiplication TokTy::Div exponentiation { e0 / e1 }
        ;

    exponentiation ::=
        | atomic TokTy::Exp exponentiation { i32::pow(e0, e1) }
        ;

    atomic ::=
        | TokTy::IntLit { to_i32(e0) }
        | TokTy::LP expr TokTy::RP { e1 }
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
