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
        action: Block,
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

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        // TODO
        let e = input.parse()?;
        Ok(Node::Literal(e))
    }
}
