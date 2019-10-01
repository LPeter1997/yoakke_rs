/**
 * Structures and traits for a lexer.
 */

use std::marker::PhantomData;
use std::ops::Range;
use std::convert::TryFrom;
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
    already_ended: bool,
    phantom: PhantomData<T>,
}

impl <'a, T> Iter<'a, T> {
    fn with_source_and_state(source: &'a str, state: LexerState) -> Self {
        Self{ source, state, already_ended: false, phantom: PhantomData, }
    }

    fn with_source(source: &'a str) -> Self {
        Self::with_source_and_state(source, LexerState::new())
    }
}

impl <'a, T> Iterator for Iter<'a, T> where T : TokenType {
    type Item = Token<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match T::next_lexeme_internal(self.source, &self.state) {
                (state, Some(kind), mut lookahead) => {
                    let range = self.state.source_index..state.source_index;
                    lookahead -= range.end;
                    let position = self.state.position;
                    self.state = state;
                    // If it's the end and we have already returned that, stop iteration
                    if kind.is_end() {
                        if self.already_ended {
                            return None;
                        }
                        else {
                            self.already_ended = true;
                            return Some(Token{ range, kind, position, lookahead });
                        }
                    }
                    else {
                        return Some(Token{ range, kind, position, lookahead });
                    }
                },

                (state, None, _) => {
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
    pub erased: Range<usize>,
    pub inserted: Vec<Token<T>>,
    pub offset: isize,
}

/**
 * The builtin lexer.
 */

pub struct StandardLexer<T> {
    source: String,
    phantom: PhantomData<T>,
}

impl <T> StandardLexer<T> where T : PartialEq {
    pub fn new() -> Self {
        Self{ source: String::new(), phantom: PhantomData, }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    fn invalidated_range(tokens: &[Token<T>], erased: &Range<usize>) -> Range<usize> {
        let mut lower = match tokens.binary_search_by_key(&erased.start, |t| t.range.end + t.lookahead) {
            Ok(idx) | Err(idx) => idx,
        };
        let mut upper = match tokens[lower..].binary_search_by_key(&erased.end, |t| t.range.end) {
            Ok(idx) | Err(idx) => idx,
        } + lower;

        if lower > 0 {
            lower -= 1;
        }
        if upper < tokens.len() {
            upper += 1;
        }

        lower..upper
    }

    fn to_isize_range(r: &Range<usize>) -> Range<isize> {
        isize::try_from(r.start).unwrap()..isize::try_from(r.end).unwrap()
    }

    fn equivalent_tokens(t1: &Token<T>, t2: &Token<T>, offs2: isize) -> bool {
        let r1 = Self::to_isize_range(&t1.range);
        let r2 = Self::to_isize_range(&t2.range);
        let r2 = (r2.start + offs2)..(r2.end + offs2);

        r1 == r2 && t1.lookahead == t2.lookahead && t1.kind == t2.kind
    }
}

impl <T> Lexer for StandardLexer<T> where T : TokenType + PartialEq {
    type TokenTag = T;

    fn iter(&self) -> Iter<Self::TokenTag> {
        Iter::with_source(&self.source)
    }

    // TODO: The actual lexing sould happen when the returned Modification is dropped
    // Similar to Drain iterator
    fn modify(&mut self, tokens: &[Token<Self::TokenTag>], erased: Range<usize>, inserted: &str)
        -> Modification<Self::TokenTag> {

        // Modify the source string
        // TODO: We could splice here
        let erased_start = erased.start;
        self.source.drain(erased.clone());
        self.source.insert_str(erased_start, inserted);

        // 'invalid' is the range of tokens that are definitely affected and removed
        // This doesn't necessarily mean that this will be the only removed range
        // as overriding tokens after that is still possible
        let mut invalid = Self::invalidated_range(tokens, &erased);
        // How much the characters shifted from the source change
        let offset = isize::try_from(inserted.len()).unwrap() - isize::try_from(erased.len()).unwrap();

        // We start from the beginning of invalid
        // We reconstruct a lexer state for that and start lexing until we are past the
        // end of invalid territory and found an equivalent token
        // If we are past the invalidation point but we find no equivalent token,
        // we need to modify the invalidation range to include that token.

        // Construct an initial state
        let start_state = if invalid.start > 0 {
            let last_tok = &tokens[invalid.start - 1];
            let last_idx = last_tok.range.start;
            LexerState{
                source_index: last_idx,
                position: last_tok.position,
                last_char: self.source[..last_idx].chars().rev().next(),
            }
        }
        else {
            LexerState::new()
        };

        // The index where we can count on equivalent state
        let last_insertion = erased.start + inserted.len();

        let mut inserted = Vec::new();

        // Now we go until we hit an equivalent state
        let mut it = Iter::<T>::with_source_and_state(&self.source, start_state);
        while let Some(token) = it.next() {
            if token.range.start > last_insertion {
                // Possibly an equivalent state
                if invalid.end < tokens.len() {
                    // Compare tokens
                    let existing = &tokens[invalid.end];
                    if token.range.end <= existing.range.start {
                        // We just insert, the new token is completely before the existing one
                        inserted.push(token);
                    }
                    else {
                        // The new token is after or intersects with the existing one
                        // We need to check for equivalence
                        // If equivalent, we are done
                        // If not equivalent, we need to erase that token
                        if Self::equivalent_tokens(existing, &token, offset) {
                            // Equivalent, we are done
                            break;
                        }
                        else {
                            // Not equivalent, erase, insert
                            invalid.end += 1;
                            inserted.push(token);
                        }
                    }
                }
                else {
                    // We are inserting at the end
                    inserted.push(token);
                }
            }
            else {
                // No possibility of an ewuivalent state, just insert
                inserted.push(token);
            }
        }

        Modification{ erased: invalid, inserted, offset }
    }
}
