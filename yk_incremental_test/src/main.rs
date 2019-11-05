extern crate yk_lexer;
extern crate yk_parser;

use std::io::{self, BufRead, Read, Bytes};
use std::time::{Duration, Instant};
use yk_lexer::{Lexer, TokenType, Token};
use yk_parser::{ParseResult, ParseErr, Found};

#[derive(Lexer, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tok {
    #[error] Error,
    #[end] EndOfInput,

    #[regex(r"[ \r\n\t]")] #[ignore] Ws,

    #[regex(r"[0-9]+")] IntLit,
    #[c_ident] Ident,

    #[token("or")] Or,
    #[token("and")] And,
    #[token("<")] Lt,
    #[token("<=")] Le,
    #[token(">")] Gt,
    #[token(">=")] Ge,
    #[token("==")] Eq,
    #[token("~=")] Neq,
    #[token("..")] Cat,
    #[token("+")] Add,
    #[token("-")] Sub,
    #[token("*")] Mul,
    #[token("/")] Div,
    #[token("%")] Mod,
    #[token("not")] Not,
    #[token("#")] Hash,
    #[token("^")] Exp,

    #[token("=")] Asgn,

    #[token(";")] Sc,
    #[token(".")] Dot,
    #[token(",")] Comma,

    #[token("...")] Ellipsis,

    #[token("function")] Function,
    #[token("do")] Do,
    #[token("while")] While,
    #[token("repeat")] Repeat,
    #[token("until")] Until,
    #[token("if")] If,
    #[token("else")] Else,
    #[token("elseif")] ElseIf,
    #[token("for")] For,
    #[token("end")] End,
    #[token("break")] Break,
    #[token("return")] Return,
    #[token("local")] Local,

    #[token("nil")] Nil,
    #[token("true")] True,
    #[token("false")] False,

    #[token("(")] LeftParen,
    #[token(")")] RightParen,

    #[token("[")] LeftBracket,
    #[token("]")] RightBracket,

    #[token("{")] LeftBrace,
    #[token("}")] RightBrace,
}

mod lua {
    use std::convert::TryInto;
    use crate::Tok;
    use yk_lexer::Token;
    use yk_parser::yk_parser;

    yk_parser!{
        item = Token<Tok>;

        type = ();

        // The lua grammar based on http://lua-users.org/wiki/LuaGrammar

        prg          ::= block $end { };

        semi         ::=
            | ";" { }
            | $epsilon
            ;

        block        ::=
            | scope statlist { }
            | scope statlist laststat semi { }
            ;

        ublock       ::= block "until" exp { };

        scope        ::=
            | scope statlist binding semi { }
            | $epsilon
            ;

        statlist     ::=
            | statlist stat semi { }
            | $epsilon
            ;

        stat         ::=
            | "do" block "end" { }
            | "while" exp "do" block "end" { }
            | repetition "do" block "end" { }
            | "repeat" ublock { }
            | "if" conds "end" { }
            | "function" funcname funcbody { }
            | setlist "=" explist1 { }
            | functioncall { }
            ;

        repetition   ::=
            | "for" Tok::Ident "=" explist23 { }
            | "for" namelist "in" explist1 { }
            ;

        conds        ::=
            | condlist { }
            | condlist "else" block { }
            ;
        condlist     ::=
            | cond { }
            | condlist "elseif" cond { }
            ;
        cond         ::= exp "then" block { };

        laststat     ::=
            | "break" { }
            | "return" { }
            | "return" explist1 { }
            ;

        binding      ::=
            | "local" namelist { }
            | "local" namelist "=" explist1 { }
            | "local" "function" Tok::Ident funcbody { }
            ;

        funcname     ::=
            | dottedname { }
            | dottedname ":" Tok::Ident { }
            ;

        dottedname   ::=
            | Tok::Ident { }
            | dottedname "." Tok::Ident { }
            ;

        namelist     ::=
            | Tok::Ident { }
            | namelist "," Tok::Ident { }
            ;

        explist1     ::=
            | exp { }
            | explist1 "," exp { }
            ;
        explist23    ::=
            | exp "," exp { }
            | exp "," exp "," exp { }
            ;

        exp          ::= exp0 { };

        exp0         ::=
            | exp0 "or" exp1 { }
            | exp1 { }
            ;
        exp1         ::=
            | exp1 "and" exp2 { }
            | exp2 { }
            ;
        exp2         ::=
            | exp2 ("<" | "<=" | ">" | ">=" | "==" | "~=") exp3 { }
            | exp3 { }
            ;
        exp3         ::=
            | exp4 ".." exp3  { }
            | exp4 { }
            ;
        exp4         ::=
            | exp4 ("+" | "-") exp5 { }
            | exp5 { }
            ;
        exp5         ::=
            | exp5 ("*" | "/" | "%") exp6 { }
            | exp6 { }
            ;
        exp6         ::=
            | "not" exp6 { }
            | "#" exp6 { }
            | exp7 { }
            ;
        exp7         ::=
            | exp8 "^" exp7 { }
            | exp8 { }
            ;
        exp8         ::=
            | ("nil" | "true" | "false" | Tok::IntLit | "...") { }
            | function { }
            | prefixexp { }
            | tableconstructor { }
            ;

        setlist      ::=
            | var { }
            | setlist "," var { }
            ;

        var          ::=
            | Tok::Ident { }
            | prefixexp "[" exp "]" {  }
            | prefixexp "." Tok::Ident { }
            ;

        prefixexp    ::=
            | var { }
            | functioncall { }
            | "(" exp ")" { }
            ;

        functioncall ::=
            | prefixexp args { }
            | prefixexp ":" Tok::Ident args { }
            ;

        args         ::=
            | "(" ")" { }
            | "(" explist1 ")" { }
            | tableconstructor { }
            ;

        function     ::= "function" funcbody { };
        funcbody     ::= params block "end" { };
        params       ::= "(" parlist ")" { };

        parlist      ::=
            | namelist { }
            | "..." { }
            | namelist "," "..." { }
            | $epsilon
            ;

        tableconstructor ::=
            | "{" "}" { }
            | "{" fieldlist "}" { }
            | "{" fieldlist ("," | ";") "}" { }
            ;

        fieldlist    ::=
            | field { }
            | fieldlist ("," | ";") field { }
            ;

        field        ::=
            | exp { }
            | Tok::Ident "=" exp { }
            | "[" exp "]" "=" exp { }
            ;
    }

    impl Match<EndOfInput> for Parser {
        fn matches(a: &Token<Tok>, b: &EndOfInput) -> bool {
            a.kind == Tok::EndOfInput
        }

        fn show_expected(t: &EndOfInput) -> String {
            "end of input".into()
        }
    }

    impl Match<Tok> for Parser {
        fn matches(a: &Token<Tok>, b: &Tok) -> bool {
            a.kind == *b
        }

        fn show_expected(t: &Tok) -> String {
            format!("{:?}", t)
        }
    }

    impl Match<&str> for Parser {
        fn matches(a: &Token<Tok>, b: &&str) -> bool {
            a.value == *b
        }

        fn show_expected(t: &&str) -> String {
            (*t).into()
        }
    }
}

fn dump_error(err: &ParseErr<Token<Tok>>) {
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

fn res_to_str(res: ParseResult<(), Token<Tok>>) -> String {
    if res.is_err() {
        dump_error(&res.err().unwrap());
        "err".into()
    }
    else {
        "ok".into()
    }
}

fn parse_number(i: &mut Bytes<impl Read>) -> usize {
    let mut n: usize = 0;
    let mut has_parsed = false;
    while let Some(res) = i.next() {
        has_parsed = true;
        let dig = res.unwrap();
        if dig == ';' as u8 {
            break;
        }
        n = n * 10 + ((dig - ('0' as u8)) as usize);
    }
    if !has_parsed {
        std::process::exit(0);
    }
    n
}

fn parse_content(i: &mut Bytes<impl Read>, len: usize) -> String {
    let mut v = Vec::new();
    for _ in 0..len {
        if let Some(res) = i.next() {
            let c = res.unwrap();
            v.push(c);
        }
        else {
            break;
        }
    }
    String::from_utf8(v).unwrap()
}

fn main() {
    let mut nonincr_source = String::new();

    let mut incr_tokens = Vec::new();
    let mut incr_lexer = Tok::lexer();
    let mut incr_parser = lua::Parser::new();

    let stdin = io::stdin();
    let stdin_lock = stdin.lock();
    let mut bytes = stdin_lock.bytes();
    loop {
        let offset = parse_number(&mut bytes);
        let removed = parse_number(&mut bytes);
        let inserted = parse_number(&mut bytes);
        let inserted_content = parse_content(&mut bytes, inserted);

        // Skip newline
        bytes.next();
        bytes.next();

        let removed_range = offset..(offset + removed);

        // Nonincremental
        let nir = {
            let mut lexer = Tok::lexer();
            let mut tokens = Vec::new();
            let mut parser = lua::Parser::new();

            let start = Instant::now();

            nonincr_source.drain(removed_range.clone());
            nonincr_source.insert_str(offset, &inserted_content);

            let m = lexer.modify(&tokens, 0..0, &nonincr_source);
            tokens.splice(m.erased, m.inserted);

            let r = parser.prg(tokens.iter().cloned());

            let elapsed = start.elapsed();
            println!("Full-parse took: {}", elapsed.as_millis());

            res_to_str(r)
        };

        let ir = {
            let start = Instant::now();

            let m = incr_lexer.modify(&incr_tokens, removed_range, &inserted_content);
            let m = m.apply(&mut incr_tokens);

            incr_parser.invalidate(m.erased, m.inserted);

            let r = incr_parser.prg(incr_tokens.iter().cloned());

            let elapsed = start.elapsed();
            println!("Incremental-parse took: {}", elapsed.as_millis());

            res_to_str(r)
        };

        println!("{} == {}", nir, ir);
    }
}
