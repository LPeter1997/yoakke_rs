
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

    // TODO: We could make the macro able to change types midway, like
    // type: xyz;
    // rules...
    // type: ijk;
    // rules...
    // That would make writing blocks of grammar easier

    yk_parser!{
        item = Token<TokTy>;

        // Statements
        type = Box<Stmt>;

        program ::= compound_stmt;

        stmt ::=
            | if_stmt
            | while_stmt
            | asgn_stmt
            | print_stmt
            | fn_stmt
            | ret_stmt
            | "{" compound_stmt "}" { e1 }
            ;

        if_stmt ::=
            | "if" expr stmt "else" stmt { Box::new(Stmt::If(e1, e2, Some(e4))) }
            | "if" expr stmt { Box::new(Stmt::If(e1, e2, None)) }
            ;

        while_stmt ::=
            | "while" expr stmt { Box::new(Stmt::While(e1, e2)) }
            ;

        asgn_stmt ::=
            | TokTy::Ident "=" expr { Box::new(Stmt::Asgn(e0.value.clone(), e2)) }
            ;

        compound_stmt ::=
            | compound_stmt stmt { if let Stmt::Compound(mut ss) = *e0 { ss.push(e1); Box::new(Stmt::Compound(ss)) } else { panic!("No"); } }
            | stmt { Box::new(Stmt::Compound(vec![e0])) }
            ;

        print_stmt ::=
            | "print" expr { Box::new(Stmt::Print(e1)) }
            ;

        fn_stmt ::=
            | "fn" TokTy::Ident "(" param_list ")" "{" compound_stmt "}" { Box::new(Stmt::Fn(e1.value.clone(), e3, e6)) }
            | "fn" TokTy::Ident "{" compound_stmt "}" { Box::new(Stmt::Fn(e1.value.clone(), Vec::new(), e3)) }
            ;

        param_list[Vec<String>] ::=
            | param_list TokTy::Ident { let mut e0 = e0; e0.push(e1.value.clone()); e0 }
            | TokTy::Ident { vec![e0.value.clone()] }
            ;

        ret_stmt ::= "return" expr { Box::new(Stmt::Return(e1)) };

        // Expressions
        type = Box<Expr>;

        expr ::= or_expr;

        or_expr ::=
            | or_expr "or" and_expr { Box::new(Expr::Or(e0, e2)) }
            | and_expr
            ;

        and_expr ::=
            | and_expr "and" eq_expr { Box::new(Expr::And(e0, e2)) }
            | eq_expr
            ;

        eq_expr ::=
            | eq_expr "==" rel_expr { Box::new(Expr::Eq(e0, e2)) }
            | eq_expr "!=" rel_expr { Box::new(Expr::Neq(e0, e2)) }
            | rel_expr
            ;

        rel_expr ::=
            | rel_expr ">" add_expr { Box::new(Expr::Gr(e0, e2)) }
            | rel_expr "<" add_expr { Box::new(Expr::Le(e0, e2)) }
            | rel_expr ">=" add_expr { Box::new(Expr::GrEq(e0, e2)) }
            | rel_expr "<=" add_expr { Box::new(Expr::LeEq(e0, e2)) }
            | add_expr
            ;

        add_expr ::=
            | add_expr "+" mul_expr { Box::new(Expr::Add(e0, e2)) }
            | add_expr "-" mul_expr { Box::new(Expr::Sub(e0, e2)) }
            | mul_expr
            ;

        mul_expr ::=
            | mul_expr "*" unary_expr { Box::new(Expr::Mul(e0, e2)) }
            | mul_expr "/" unary_expr { Box::new(Expr::Div(e0, e2)) }
            | mul_expr "%" unary_expr { Box::new(Expr::Mod(e0, e2)) }
            | unary_expr
            ;

        unary_expr ::=
            | "!" unary_expr { Box::new(Expr::Not(e1)) }
            | "-" unary_expr { Box::new(Expr::Neg(e1)) }
            | atom
            ;

        atom ::=
            | TokTy::IntLit { Box::new(Expr::IntLit(e0.value.parse::<i32>().unwrap())) }
            | TokTy::Ident { Box::new(Expr::Ident(e0.value.clone())) }
            | "(" expr ")" { e1 }
            ;
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

    impl ShowFound for Parser {
        fn show_found(t: &Token<TokTy>) -> String {
            t.value.clone()
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

fn main() {
    let src = "
fn foo(x) { y =  }

while 1 {
    n = read

    is_prime = 1
    i = 2
    while i < n {
        if n % i == 0 {
            is_prime = 0
        }
        i = i + 1
    }
    if n == 1 {
        is_prime = 0
    }
    print is_prime
}
    ";
    //let stdin = io::stdin();
    //for line in stdin.lock().lines() {

    let mut lexer = TokTy::lexer();
    let mut tokens = Vec::new();

    //let line = &line.unwrap();
    let m = lexer.modify(&tokens, 0..0, src);
    tokens.splice(m.erased, m.inserted);

    let mut parser = peg::Parser::new();
    let r = parser.program(tokens.iter().cloned());
    if r.is_ok() {
        let ok = r.ok().unwrap();
        let val = ok.value;
        let mlen = ok.matched;
        println!("Parse succeeded, matched: {}", mlen);
        let mut interpr = Interpreter::new();
        interpr.push_stack();
        interpr.execute(&val);
        interpr.pop_stack();
        //println!("Ok: {:?} (matched: {})", val, mlen);
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

    //}
}
