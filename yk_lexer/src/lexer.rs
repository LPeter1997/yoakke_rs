/**
 * Structures and traits for a lexer.
 */

use crate::position::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexerState {
    pub source_index: usize,
    pub position: Position,
}

pub trait Lexer<T> {
    fn new() -> Self;
}
