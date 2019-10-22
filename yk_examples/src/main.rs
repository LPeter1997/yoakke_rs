
extern crate yk_lexer;
extern crate yk_parser;

use yk_lexer::{TokenType, Lexer};
use yk_parser::{yk_parser, ParseResult};

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
name: MyParser;
type: i32;

ones ::=
    | ones_impl
    ;

ones_impl ::=
    | ones '1' { e0 + 1 }
    | '1' { 1 }
    ;
}

fn main() {
    let src = "1111";

    let r = parser::parse_ones(&mut parser::MemoContext::new(), src.chars(), 0);
    if r.is_ok() {
        let ok = r.ok().unwrap();
        let val = ok.value;
        let mlen = ok.matched;
        println!("Ok: {:?} (matched: {})", val, mlen);
    }
    else {
        let err = r.err().unwrap();
        println!("Err:");
        for (rule, element) in err.elements {
            print!("  While parsing {} expected: ", rule);

            let mut fst = true;
            for tok in element.expected_elements {
                if !fst {
                    print!(" or ");
                }
                fst = false;
                print!("{}", tok);
            }
            println!();
        }
        println!("But got '{}'", err.found_element);
    }

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
