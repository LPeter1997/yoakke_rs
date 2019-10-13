/**
 * Helper structure for direct left-recursion.
 */

use crate::parse_result::ParseResult;

pub enum DirectRec<I, T> {
    Base(ParseResult<I, T>, bool),
    Recurse(ParseResult<I, T>),
}
