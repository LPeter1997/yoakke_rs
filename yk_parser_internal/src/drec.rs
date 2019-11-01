/**
 * Helper structure for direct left-recursion.
 */

use crate::parse_result::ParseResult;

pub enum DirectRec<T, E> {
    Base(ParseResult<T, E>),
    Stub(ParseResult<T, E>),
    Recurse(ParseResult<T, E>),
}

impl <T, E> DirectRec<T, E> {
    pub fn parse_result(&self) -> &ParseResult<T, E> {
        match self {
            DirectRec::Base(res) => res,
            DirectRec::Stub(res) => res,
            DirectRec::Recurse(res) => res,
        }
    }
}
