/**
 * Token definition.
 */

use std::ops::Range;
use crate::position::Position;
use crate::lexer::{LexerState, Lexer, BuiltinLexer};

/// A generic token that's being returned by the lexer.
pub struct Token<T> {
    pub range: Range<usize>,
    pub kind: T,
    pub position: Position,
}

/// The type that the derive-macro implements on the user-defined enum.
/// This is where the actual lexer logic is injected.
pub trait TokenType : Sized {
    fn lexer() -> BuiltinLexer<Self>;
    fn next_lexeme_internal(src: &str, state: LexerState) -> (LexerState, Option<Token<Self>>);
}
