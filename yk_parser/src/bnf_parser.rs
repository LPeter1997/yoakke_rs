/**
 * Parsing BNF notation into an AST.
 */

use syn::TokenStream;
use crate::bnf_ast::{RuleSet, Node};

/**
 * The allowed syntax is:
 *
 * expr ::= foo bar baz { construct(x0, x1, x2) }
 *        | qux uho wat { /* ... */ }
 *        ;
 */

pub fn parse(tokens: TokenStream) -> RuleSet {

}
