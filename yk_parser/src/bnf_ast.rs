/**
 * Backus–Naur form AST representation.
 */

use std::collections::HashMap;
use syn::Expr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSet {
    pub rules: HashMap<String, Node>,
    pub literal_matcher: (),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Alternative{
        first: Box<Node>,
        second: Box<Node>,
    },

    Sequence{
        first: Box<Node>,
        second: Box<Node>,
    },

    Transformation {
        subnode: Box<Node>,
    },

    Literal(Expr),
}
