/**
 * Token definition.
 */

use std::convert::TryFrom;
use std::ops::Range;
use crate::position::Position;
use crate::lexer::{LexerState, StandardLexer};

/// A generic token that's being returned by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<T> {
    pub range: Range<usize>,
    pub kind: T,
    pub position: Position,
    pub lookahead: usize,
}

impl <T> Token<T> {
    pub fn shift(&mut self, offset: isize) {
        let start = self.range.start;
        let start = isize::try_from(start).unwrap() + offset;
        let start = usize::try_from(start).unwrap();

        let end = self.range.end;
        let end = isize::try_from(end).unwrap() + offset;
        let end = usize::try_from(end).unwrap();

        self.range.start = start;
        self.range.end = end;
    }
}

/// The type that the derive-macro implements on the user-defined enum.
/// This is where the actual lexer logic is injected.
pub trait TokenType : Sized {
    fn lexer() -> StandardLexer<Self> where Self : PartialEq {
        StandardLexer::new()
    }

    fn is_end(&self) -> bool;
    fn next_lexeme_internal(src: &str, state: &LexerState) -> (LexerState, Option<Self>, usize);
}
