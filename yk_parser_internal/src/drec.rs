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
              DirectRec::Base(res)
            | DirectRec::Stub(res)
            | DirectRec::Recurse(res) => res,
        }
    }

    pub fn furthest_look(&self) -> usize {
        self.parse_result().furthest_look()
    }
}
