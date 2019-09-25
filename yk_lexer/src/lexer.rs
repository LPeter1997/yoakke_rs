/**
 * Structures and traits for a lexer.
 */

use std::ops::Range;
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
    pub fn new() -> Self {
        Self{ source_index: 0, position: Position::new(), last_char: None, }
    }
}

/**
 * Lexer interface and implementation.
 */

pub trait Lexer<T> where T : TokenType {
    fn modify(&mut self, tokens: &[T], erased: Range<usize>, inserted: &str) -> Modification<T>;
}

impl <L, T> Iterator for &L where L : Lexer<T>, T : TokenType {

}

/**
 * Modification descriptions.
 */

pub struct Modification<T> {
    erasure: Erasure<T>,
    insertion: Insertion<T>,
}

pub struct Erasure<T> {
    range: Range<usize>,
    phantom: PhantomData<T>,
}

pub struct Insertion<T> {
    // TODO
    phantom: PhantomData<T>,
}

/**
 * The builtin lexer.
 */

pub struct StandardLexer<T> {
    source: String,
    state: LexerState,
    phantom: PhantomData<T>,
}

impl <T> StandardLexer<T> {
    pub fn new() -> Self {
        Self{ source: String::new(), state: LexerState::new(), phantom: PhantomData, }
    }
}
