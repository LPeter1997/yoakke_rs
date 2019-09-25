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

pub trait Lexer {
    type TokenTag : TokenType;

    fn iter(&self) -> Iter<Self::TokenTag>;

    fn modify(&mut self, tokens: &[Token<Self::TokenTag>], erased: Range<usize>, inserted: &str)
        -> Modification<Self::TokenTag>;
}

/**
 * Iterate over all tokens.
 */

pub struct Iter<'a, T> {
    source: &'a str,
    state: LexerState,
    phantom: PhantomData<T>,
    already_ended: bool,
}

impl <'a, T> Iter<'a, T> {
    fn with_source(source: &'a str) -> Self {
        Self{ source, state: LexerState::new(), phantom: PhantomData, already_ended: false, }
    }
}

impl <'a, T> Iterator for Iter<'a, T> where T : TokenType {
    type Item = Token<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match T::next_lexeme_internal(self.source, &self.state) {
                (state, Some(kind)) => {
                    let range = self.state.source_index..state.source_index;
                    let position = self.state.position;
                    self.state = state;
                    // If it's the end and we have already returned that, stop iteration
                    if kind.is_end() {
                        if self.already_ended {
                            return None;
                        }
                        else {
                            self.already_ended = true;
                            return Some(Token{ range, kind, position });
                        }
                    }
                    else {
                        return Some(Token{ range, kind, position });
                    }
                },

                (state, None) => {
                    // Ignored token
                    self.state = state;
                }
            }
        }
    }
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

    pub fn source(&self) -> &str {
        &self.source
    }

    fn invalidated_range(tokens: &[Token<T>], erased: &Range<usize>) -> Range<usize> {
        let mut lower = match tokens.binary_search_by_key(&erased.start, |t| t.range.start) {
            Ok(idx) | Err(idx) => idx,
        };
        let upper = match tokens[lower..].binary_search_by_key(&erased.end, |t| t.range.end) {
            Ok(idx) => idx + 1,
            Err(idx) => idx,
        } + lower;
        if lower > 0 {
            lower -= 1;
        }
        lower..upper
    }
}

impl <T> Lexer for StandardLexer<T> where T : TokenType {
    type TokenTag = T;

    fn iter(&self) -> Iter<Self::TokenTag> {
        Iter::with_source(&self.source)
    }

    fn modify(&mut self, tokens: &[Token<Self::TokenTag>], erased: Range<usize>, inserted: &str)
        -> Modification<Self::TokenTag> {

        // Modify the source string
        // TODO: We could splice here
        let erased_start = erased.start;
        self.source.drain(erased.clone());
        self.source.insert_str(erased_start, inserted);

        let mut invalid = Self::invalidated_range(tokens, &erased);
        // TODO

        // TODO
        Modification{ erasure: Erasure{ range: 0..0, phantom: PhantomData }, insertion: Insertion{ phantom: PhantomData } }
    }
}
