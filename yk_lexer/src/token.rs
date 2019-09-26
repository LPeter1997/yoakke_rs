/**
 * Token definition.
 */

use std::ops::Range;
use crate::position::Position;
use crate::lexer::{LexerState, StandardLexer};

/// A generic token that's being returned by the lexer.
pub struct Token<T> {
    pub range: Range<usize>,
    pub kind: T,
    pub position: Position,
    pub lookahead: usize,
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
