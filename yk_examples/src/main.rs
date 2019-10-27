
extern crate yk_lexer;
extern crate yk_parser;

use std::io::{self, BufRead};
use std::collections::HashMap;
use yk_lexer::{Token, TokenType, Lexer};
use yk_parser::{yk_parser, ParseResult, Match};

#[derive(Lexer, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokTy {
    #[error] Error,
    #[end] End,
    #[regex(r"[ \r\n]")] #[ignore] Whitespace,

    #[c_ident] Ident,
    #[regex(r"[0-9]+")] IntLit,

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

    #[token("!")] Not,
    #[token("and")] And,
    #[token("or")] Or,

    #[token("(")] LeftParen,
    #[token(")")] RightParen,

    #[token("=")] Asgn,

    // TODO: Builds without these
    // But there's a weird range (larger..=smaller => ...)
    // in Ident matching status when these are here
    /*#[token("if")] KwIf,
    #[token("else")] KwElse,
    #[token("while")] KwWhile,
    #[token("print")] KwPrint,*/

    #[token("{")] LeftBrace,
    #[token("}")] RightBrace,
}

mod peg {
    use crate::TokTy;
    use yk_parser::yk_parser;
    use yk_lexer::Token;

    fn btoi(b: bool) -> i32 { if b { 1 } else { 0 } }
    fn itob(i: i32) -> bool { i != 0 }

    yk_parser!{
        item: Token<TokTy>;
        type: i32;

        expr ::= or_expr;

        or_expr ::=
            | or_expr "or" and_expr { btoi(itob(e0) || itob(e2)) }
            | and_expr
            ;

        and_expr ::=
            | and_expr "and" eq_expr { btoi(itob(e0) && itob(e2)) }
            | eq_expr
            ;

        eq_expr ::=
            | eq_expr "==" rel_expr { btoi(e0 == e2) }
            | eq_expr "!=" rel_expr { btoi(e0 != e2) }
            | rel_expr
            ;

        rel_expr ::=
            | rel_expr ">" add_expr { btoi(e0 > e2) }
            | rel_expr "<" add_expr { btoi(e0 < e2) }
            | rel_expr ">=" add_expr { btoi(e0 >= e2) }
            | rel_expr "<=" add_expr { btoi(e0 <= e2) }
            | add_expr
            ;

        add_expr ::=
            | add_expr "+" mul_expr { e0 + e2 }
            | add_expr "-" mul_expr { e0 - e2 }
            | mul_expr
            ;

        mul_expr ::=
            | mul_expr "*" unary_expr { e0 * e2 }
            | mul_expr "/" unary_expr { e0 / e2 }
            | unary_expr
            ;

        unary_expr ::=
            | "!" unary_expr { btoi(!itob(e1)) }
            | "-" unary_expr { -e1 }
            | atom
            ;

        atom ::=
            | TokTy::IntLit { e0.value.parse::<i32>().unwrap() }
            | "(" expr ")" { e1 }
            ;
    }

    impl Match<TokTy> for Parser {
        fn matches(a: &Token<TokTy>, b: &TokTy) -> bool {
            a.kind == *b
        }

        fn show_expected(t: &TokTy) -> String {
            "<TokTy>".into()
        }
    }

    impl Match<&str> for Parser {
        fn matches(a: &Token<TokTy>, b: &&str) -> bool {
            a.value == *b
        }

        fn show_expected(t: &&str) -> String {
            (*t).into()
        }
    }

    impl ShowFound for Parser {
        fn show_found(t: &Token<TokTy>) -> String {
            t.value.clone()
        }
    }
}

pub enum Stmt {
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    Asgn(String, Box<Expr>),
    Compound(Vec<Box<Stmt>>),
    Expr(Box<Expr>),
    Print(Box<Expr>),
}

pub enum Expr {
    Ident(String),
    IntLit(i32),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    Gr(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    GrEq(Box<Expr>, Box<Expr>),
    LeEq(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Not(Box<Expr>),
}

struct Interpreter {
    variables: Vec<HashMap<String, i32>>,
}

impl Interpreter {
    fn new() -> Self {
        Self{
            variables: Vec::new(),
        }
    }

    fn execute(&mut self, stmt: Box<Stmt>) {
        unimplemented!();
    }

    fn evaluate(&mut self, expr: Box<Expr>) -> i32 {
        unimplemented!();
    }

    fn ref_var(&mut self, name: &String) -> i32 {
        for sc in self.variables.iter().rev() {
            if let Some(v) = sc.get(name) {
                return *v;
            }
        }
        panic!("Undefined variable {}!", name);
    }

    fn set_var(&mut self, name: &String, val: i32) {
        self.variables.last_mut().unwrap().insert(name.clone(), val);
    }

    fn push_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.variables.pop();
    }
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {

    let mut lexer = TokTy::lexer();
    let mut tokens = Vec::new();

    let m = lexer.modify(&tokens, 0..0, &line.unwrap());
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

    }
}
