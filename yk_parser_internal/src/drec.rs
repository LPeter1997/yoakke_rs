/**
 * Helper structure for direct left-recursion.
 */

use crate::parse_result::ParseResult;

pub enum DirectRec<I, T> {
    Base(ParseResult<I, T>, bool),
    Recurse(ParseResult<I, T>),
}

impl <I, T> DirectRec<I, T> {
    pub fn parse_result(&self) -> &ParseResult<I, T> {
        match self {
            DirectRec::Base(res, _) => res,
            DirectRec::Recurse(res) => res,
        }
    }
}
