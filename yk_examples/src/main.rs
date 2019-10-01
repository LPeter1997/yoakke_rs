
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

#[derive(Debug)]
enum AST {
    Add(Box<AST>, Box<AST>),
    Sub(Box<AST>, Box<AST>),
    Atom(i32),
}

yk_parser!{
    addition ::=
        | atomic '+' addition { Box::new(AST::Add(e0, e2)) }
        | atomic '-' addition { Box::new(AST::Sub(e0, e2)) }
        | atomic
        ;

    atomic ::=
        | '0' { Box::new(AST::Atom(0)) }
        | '1' { Box::new(AST::Atom(1)) }
        ;
}

fn main() {
    let src = "1+1-0-1+1+0+0+0";
    let r = parse_addition(src.chars());
    println!("{:?}", r);
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
