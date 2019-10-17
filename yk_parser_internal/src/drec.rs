/**
 * Helper structure for direct left-recursion.
 */

use crate::parse_result::ParseResult;

pub enum DirectRec<I, T> {
    Base(ParseResult<I, T>),
    Stub(ParseResult<I, T>),
    Recurse(ParseResult<I, T>),
}

impl <I, T> DirectRec<I, T> {
    pub fn parse_result(&self) -> &ParseResult<I, T> {
        match self {
            DirectRec::Base(res) => res,
            DirectRec::Stub(res) => res,
            DirectRec::Recurse(res) => res,
        }
    }
}
