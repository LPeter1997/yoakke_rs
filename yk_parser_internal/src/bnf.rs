/**
 * Backus–Naur form AST representation and parsing.
 */

use std::collections::HashMap;
use syn::{Result, Token, Expr, Block, Ident, Lit};
use syn::parse::{Parse, Parser, ParseStream};
use syn::punctuated::Punctuated;

#[derive(Clone)]
pub struct RuleSet {
    pub rules: HashMap<String, Node>,
    pub literal_matcher: (),
}

#[derive(Clone)]
pub enum Node {
    Toplevel{
        subnode: Box<Node>,
        action: Block,
    },

    Alternative{
        first: Box<Node>,
        second: Box<Node>,
    },

    Sequence{
        first: Box<Node>,
        second: Box<Node>,
    },

    Literal(Lit),
}

/**
 * The allowed syntax is:
 *
 * expr ::= foo bar baz { construct(x0, x1, x2) }
 *        | qux uho wat { /* ... */ }
 *        ;
 */

struct Rule {
    ident: Ident,
    col: Token![::],
    eq: Token![=],
    node: Node,
}

impl Parse for RuleSet {
    fn parse(input: ParseStream) -> Result<Self> {
        let nnp = input.parse_terminated::<Rule, Token![;]>(Rule::parse)?;
        let mut rules = HashMap::new();
        for (k, v) in nnp.iter().map(|x| (x.ident.to_string(), x.node.clone())) {
            rules.insert(k, v);
        }
        Ok(RuleSet{ rules, literal_matcher: (), })
    }
}

impl Parse for Rule {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Rule{
            ident: input.parse()?,
            col: input.parse()?,
            eq: input.parse()?,
            node: input.parse()?,
        })
    }
}

impl Node {
    fn parse_toplevel(input: ParseStream) -> Result<Box<Node>> {
        unimplemented!();
    }

    fn parse_alternative(input: ParseStream) -> Result<Box<Node>> {
        unimplemented!();
    }

    fn parse_sequence(input: ParseStream) -> Result<Box<Node>> {
        unimplemented!();
    }

    fn parse_literal(input: ParseStream) -> Result<Box<Node>> {
        unimplemented!();
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_toplevel(input)
    }
}
