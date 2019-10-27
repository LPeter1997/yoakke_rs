
extern crate yk_lexer;
extern crate yk_parser;

use yk_lexer::{Token, TokenType, Lexer};
use yk_parser::{yk_parser, ParseResult, Match};

#[derive(Lexer, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokTy {
    #[error] Error,
    #[end] End,
    #[regex(r"[ \r\n]")] #[ignore] Whitespace,

    #[c_ident] Ident, // Unused in example, just for demonstration
    #[regex("[0-9]+")] IntLit,

    #[token("+")] Add,
    #[token("-")] Sub,
    #[token("*")] Mul,
    #[token("/")] Div,

    #[token(">")] Gr,
    #[token(">=")] GrEq,
    #[token("<")] Le,
    #[token("<=")] LeEq,

    #[token("==")] Eq,
    #[token("!=")] Neq,

    #[token("(")] LeftParen,
    #[token(")")] RightParen,
}

mod peg {
    use crate::TokTy;
    use yk_parser::yk_parser;
    use yk_lexer::Token;

    fn btoi(b: bool) -> i32 { if b { 1 } else { 0 } }

    yk_parser!{
        item: Token<TokTy>;
        type: i32;

        expr ::= add_expr;

        eq_expr ::=
            | eq_expr TokTy::Eq rel_expr { btoi(e0 == e2) }
            | eq_expr TokTy::Neq rel_expr { btoi(e0 != e2) }
            | rel_expr
            ;

        rel_expr ::=
            | rel_expr TokTy::Gr add_expr { btoi(e0 > e2) }
            | rel_expr TokTy::Le add_expr { btoi(e0 < e2) }
            | rel_expr TokTy::GrEq add_expr { btoi(e0 >= e2) }
            | rel_expr TokTy::LeEq add_expr { btoi(e0 <= e2) }
            | add_expr
            ;

        add_expr ::=
            | add_expr TokTy::Add mul_expr { e0 + e2 }
            | add_expr TokTy::Sub mul_expr { e0 - e2 }
            | mul_expr
            ;

        mul_expr ::=
            | mul_expr TokTy::Mul atom { e0 * e2 }
            | mul_expr TokTy::Div atom { e0 / e2 }
            | atom
            ;

        atom ::=
            | TokTy::IntLit { e0.value.parse::<i32>().unwrap() }
            | TokTy::LeftParen expr TokTy::RightParen { e1 }
            ;
    }

    impl Match<TokTy> for Parser {
        fn matches(a: &Token<TokTy>, b: &TokTy) -> bool {
            let res = a.kind == *b;
            println!("{} == {:?} => {}", a.value, b, res);
            a.kind == *b
        }
    }

    impl ShowExpected<TokTy> for Parser {
        fn show_expected(t: &TokTy) -> String {
            "<TokTy>".into()
        }
    }

    impl ShowFound for Parser {
        fn show_found(t: &Token<TokTy>) -> String {
            t.value.clone()
        }
    }
}

/*
impl <I> Match<char> for MyParser<I> where I : Iterator<Item = char> {
    fn matches(a: &char, b: &char) -> bool {
        *a == *b
    }
}
*/

fn main() {
    let mut lexer = TokTy::lexer();
    let mut tokens = Vec::new();

    let m = lexer.modify(&tokens, 0..0, "1+2");
    tokens.splice(m.erased, m.inserted);

    let mut parser = peg::Parser::new();
    let r = parser.expr(tokens.iter().cloned());
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
