/**
 * The main lexer library that connects with the derive library for the
 * derive-macro.
 */

extern crate yk_lexer_derive;

pub use yk_lexer_derive::Lexer;

pub struct Token<'a, T> {
    pub value: &'a str,
    pub kind: T,
}

pub trait TokenType<T> {
    fn with_source(source: &str) -> Self;
}

pub trait Lexer<T> {
    fn next_token(&mut self) -> Token<T>;
}

pub struct BuiltinLexer<'a, T, F> where F : FnMut(&'a str) -> (usize, Token<T>) {
    source: &'a str,
    source_slice: &'a str,
    func: F,
}

impl <'a, T, F> BuiltinLexer<'a, T, F> where F : FnMut(&'a str) -> (usize, Token<T>) {
    pub fn with_source_and_fn(source: &'a str, func: F) -> Self {
        Self{ source, source_slice: source, func, }
    }
}

impl <'a, T, F> Lexer<T> for BuiltinLexer<'a, T, F> where F : FnMut(&'a str) -> (usize, Token<T>) {
    fn next_token(&mut self) -> Token<T> {
        let (offs, tok) = self.func(self.source_slice);
        self.source_slice = &self.source_slice[offs..];
        tok
    }
}
