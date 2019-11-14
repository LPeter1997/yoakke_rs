
extern crate yk_lexer;
extern crate yk_parser;

use std::io::{self, BufRead};
use std::collections::HashMap;
use yk_lexer::{Token, TokenType, Lexer};
use yk_parser::{yk_parser, ParseResult, ParseErr, Found, Match};

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
    #[token("%")] Mod,

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
    #[token(".")] Dot,

    #[token("if")] KwIf,
    #[token("else")] KwElse,
    #[token("while")] KwWhile,
    #[token("print")] KwPrint,

    #[token("{")] LeftBrace,
    #[token("}")] RightBrace,

    #[token(",")] Comma,
}

mod peg {
    use crate::{TokTy, Expr, Stmt};
    use yk_parser::yk_parser;
    use yk_lexer::Token;

    // TODO: Look through the generated source-spans
    // to make errors more readable

    // TODO: Clean up the result-APIs
    // Sometimes we clone and return a clone, sometimes we return refs
    // Be consistent!

    // TODO: Make lexer offset the tokens so it doesn't have to be done
    // explicitly

    yk_parser!{
        item = Token<TokTy>;

        // Statements
        type = ();

        program ::= call $end { };

        call ::= ident "(" ")" {  };

        ident ::=
            | TokTy::Ident {  }
            | prefix "." TokTy::Ident { }
            ;

        prefix ::=
            | ident { }
            | ident2 { }
            | call { }
            ;

        ident2 ::=
            | TokTy::Ident {  }
            | prefix "." TokTy::Ident { }
            ;
    }

    impl Match<EndOfInput> for Parser {
        fn matches(a: &Token<TokTy>, b: &EndOfInput) -> bool {
            a.kind == TokTy::End
        }

        fn show_expected(t: &EndOfInput) -> String {
            "end of input".into()
        }
    }

    impl Match<TokTy> for Parser {
        fn matches(a: &Token<TokTy>, b: &TokTy) -> bool {
            a.kind == *b
        }

        fn show_expected(t: &TokTy) -> String {
            format!("{:?}", t)
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
}

#[derive(Clone)]
pub enum Stmt {
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    Asgn(String, Box<Expr>),
    Compound(Vec<Box<Stmt>>),
    Expr(Box<Expr>),
    Print(Box<Expr>),
    Fn(String, Vec<String>, Box<Stmt>),
    Return(Box<Expr>),
}

#[derive(Clone)]
pub enum Expr {
    Ident(String),
    IntLit(i32),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
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

struct StackFrame {
    variables: Vec<HashMap<String, i32>>,
}

impl StackFrame {
    fn new() -> Self {
        Self{ variables: Vec::new() }
    }

    fn ref_var(&mut self, name: &String) -> Option<&mut i32> {
        for sc in self.variables.iter_mut().rev() {
            if let Some(v) = sc.get_mut(name) {
                return Some(v);
            }
        }
        return None;
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

struct Interpreter {
    stack: Vec<StackFrame>,
}

fn btoi(b: bool) -> i32 { if b { 1 } else { 0 } }
fn itob(i: i32) -> bool { i != 0 }

impl Interpreter {
    fn new() -> Self {
        Self{ stack: Vec::new() }
    }

    fn execute(&mut self, stmt: &Box<Stmt>) {
        match &**stmt {
            Stmt::If(cond, then, els) => {
                if itob(self.evaluate(&cond)) {
                    self.execute(&then);
                }
                else if let Some(els) = els {
                    self.execute(&els);
                }
            },

            Stmt::While(cond, then) => {
                while itob(self.evaluate(&cond)) {
                    self.execute(&then);
                }
            },

            Stmt::Asgn(name, val) => {
                let val = self.evaluate(&val);
                self.set_var(name, val);
            },

            Stmt::Compound(stmts) => {
                self.stack.last_mut().unwrap().push_scope();
                for s in stmts {
                    self.execute(s);
                }
                self.stack.last_mut().unwrap().pop_scope();
            },

            Stmt::Expr(expr) => {
                self.evaluate(&expr);
            },

            Stmt::Print(expr) => {
                let val = self.evaluate(&expr);
                println!("{}", val);
            },

            _ => unimplemented!(),
        }
    }

    fn evaluate(&mut self, expr: &Box<Expr>) -> i32 {
        match &**expr {
            Expr::Ident(name) => {
                if name == "read" {
                    let stdin = io::stdin();
                    let line1 = stdin.lock().lines().next().unwrap().unwrap();
                    line1.parse::<i32>().unwrap()
                }
                else {
                    self.ref_var(&name)
                }
            },
            Expr::IntLit(x) => *x,
            Expr::Add(l, r) => self.evaluate(&l) + self.evaluate(&r),
            Expr::Sub(l, r) => self.evaluate(&l) - self.evaluate(&r),
            Expr::Mul(l, r) => self.evaluate(&l) * self.evaluate(&r),
            Expr::Div(l, r) => self.evaluate(&l) / self.evaluate(&r),
            Expr::Mod(l, r) => self.evaluate(&l) % self.evaluate(&r),
            Expr::Eq(l, r) => btoi(self.evaluate(&l) == self.evaluate(&r)),
            Expr::Neq(l, r) => btoi(self.evaluate(&l) != self.evaluate(&r)),
            Expr::Gr(l, r) => btoi(self.evaluate(&l) > self.evaluate(&r)),
            Expr::Le(l, r) => btoi(self.evaluate(&l) < self.evaluate(&r)),
            Expr::GrEq(l, r) => btoi(self.evaluate(&l) >= self.evaluate(&r)),
            Expr::LeEq(l, r) => btoi(self.evaluate(&l) <= self.evaluate(&r)),
            Expr::And(l, r) => btoi(itob(self.evaluate(&l)) && itob(self.evaluate(&r))),
            Expr::Or(l, r) => btoi(itob(self.evaluate(&l)) || itob(self.evaluate(&r))),
            Expr::Neg(l) => -self.evaluate(&l),
            Expr::Not(l) => btoi(!itob(self.evaluate(&l))),

            _ => unimplemented!(),
        }
    }

    fn ref_var(&mut self, name: &String) -> i32 {
        if let Some(v) = self.stack.last_mut().unwrap().ref_var(name) {
            *v
        }
        else {
            *self.stack.first_mut().unwrap().ref_var(name).unwrap_or_else(|| panic!("Undefined variable {}!", name))
        }
    }

    fn set_var(&mut self, name: &String, val: i32) {
        if let Some(v) = self.stack.last_mut().unwrap().ref_var(name) {
            *v = val;
        }
        else if let Some(v) = self.stack.first_mut().unwrap().ref_var(name) {
            *v = val;
        }
        else {
            self.stack.last_mut().unwrap().set_var(name, val);
        }
    }

    fn push_stack(&mut self) {
        self.stack.push(StackFrame::new());
    }

    fn pop_stack(&mut self) {
        self.stack.pop();
    }
}

fn dump_error(err: &ParseErr<Token<TokTy>>) {
    println!("Err:");
    for (rule, element) in &err.elements {
        print!("  While parsing {} expected: ", rule);

        let mut fst = true;
        for tok in &element.expected_elements {
            if !fst {
                print!(" or ");
            }
            fst = false;
            print!("{}", tok);
        }
        println!();
    }
    match &err.found_element {
        Found::Element(e) => println!("But got '{}'", e.value),
        Found::EndOfInput => println!("But got end of input"),
        Found::Stub => panic!(),
    }
}

fn main() {
    let src = "x.x()";

    let mut lexer = TokTy::lexer();
    let mut tokens = Vec::new();

    let m = lexer.modify(&tokens, 0..0, src);
    tokens.splice(m.erased, m.inserted);

    let mut parser = peg::Parser::new();
    let r = parser.program(tokens.iter().cloned());
    if r.is_ok() {
        let ok = r.ok().unwrap();

        let val = ok.value;
        let mlen = ok.matched;
        println!("Parse succeeded, matched: {}", mlen);
        //let mut interpr = Interpreter::new();
        //interpr.push_stack();
        //interpr.execute(&val);
        //interpr.pop_stack();
    }
    else {
        let err = r.err().unwrap();
        dump_error(&err);
    }
}
