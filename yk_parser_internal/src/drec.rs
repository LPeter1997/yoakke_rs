/**
 * Helper structure for direct left-recursion.
 */

use crate::parse_result::ParseResult;

pub enum DirectRec<T> {
    Base(ParseResult<T>),
    Stub(ParseResult<T>),
    Recurse(ParseResult<T>),
}

impl <T> DirectRec<T> {
    pub fn parse_result(&self) -> &ParseResult<T> {
        match self {
            DirectRec::Base(res) => res,
            DirectRec::Stub(res) => res,
            DirectRec::Recurse(res) => res,
        }
    }
}
