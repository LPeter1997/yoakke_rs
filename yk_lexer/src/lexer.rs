/**
 * Structures and traits for a lexer.
 */

use std::marker::PhantomData;
use crate::position::Position;
use crate::token::{TokenType, Token};

/**
 * Every lexer's minimal information it needs to carry.
 */

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexerState {
    pub source_index: usize,
    pub position: Position,
    pub last_char: Option<char>,
}

impl LexerState {
    fn new() -> Self {
        Self{ source_index: 0, position: Position::new(), last_char: None, }
    }
}

/**
 * Lexer interface and implementation.
 */

pub trait Lexer<T> {
    fn new() -> Self;
}
