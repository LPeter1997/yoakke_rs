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
    fn with_source(source: &str) -> BuiltinLexer<Self> where Self : Sized;
}

pub trait Lexer<T> {
    fn next_token(&mut self) -> Token<T>;
}

pub trait LexerInternal<T> {
    fn next_token_internal(source: &str) -> (usize, T);
}

pub struct BuiltinLexer<'a, T> {
    source: &'a str,
    source_slice: &'a str,
    phantom: std::marker::PhantomData<T>,
}

impl <'a, T> BuiltinLexer<'a, T> {
    pub fn with_source(source: &'a str) -> Self {
        Self{ source, source_slice: source, phantom: std::marker::PhantomData }
    }
}

impl <'a, T, IL> Lexer<T> for BuiltinLexer<'a, IL> where IL : LexerInternal<T> {
    fn next_token(&mut self) -> Token<T> {
        let (offs, tok_ty) = IL::next_token_internal(self.source_slice);
        let tok = Token{
            value: &self.source_slice[0..=offs],
            kind: tok_ty,
        };
        self.source_slice = &self.source_slice[offs..];
        // Advance one
        /*if let Some((next_idx, _)) = self.source_slice[offs..].char_indices().skip(1).next() {
            self.source_slice = &self.source_slice[next_idx..];
        }*/
        tok
    }
}
