/**
 * Backus–Naur form AST representation and parsing.
 */

use std::collections::HashMap;
use syn::{Result, Token, Expr, Block, Ident, Lit, Path};
use syn::parse::{Parse, Parser, ParseStream};
use syn::punctuated::Punctuated;

#[derive(Clone)]
pub struct RuleSet {
    pub top_rule: (String, Node),
    pub rules: HashMap<String, Node>,
    pub literal_matcher: (),
}

#[derive(Clone)]
pub enum Node {
    Transformation{
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

    Literal(LiteralNode),
}

#[derive(Clone)]
pub enum LiteralNode {
    Ident(Path),
    Lit(Lit),
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
        let mut top_rule = None;
        for (k, v) in nnp.iter().map(|x| (x.ident.to_string(), x.node.clone())) {
            if top_rule.is_none() {
                top_rule = Some((k.clone(), v.clone()));
            }
            rules.insert(k, v);
        }
        Ok(RuleSet{ top_rule: top_rule.unwrap(), rules, literal_matcher: (), })
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
    fn parse_toplevel_alternative(input: ParseStream) -> Result<Box<Node>> {
        // Consume optional '|'
        let _ = input.parse::<Token![|]>();
        // Left, pipe and right
        let first = Self::parse_toplevel_sequence(input)?;
        if input.parse::<Token![|]>().is_ok() {
            let second = Self::parse_toplevel_alternative_cont(input)?;
            Ok(Box::new(Node::Alternative{ first, second, }))
        }
        else {
            Ok(first)
        }
    }

    fn parse_toplevel_alternative_cont(input: ParseStream) -> Result<Box<Node>> {
        // Left, pipe and right
        let first = Self::parse_toplevel_sequence(input)?;
        if input.parse::<Token![|]>().is_ok() {
            let second = Self::parse_toplevel_alternative_cont(input)?;
            Ok(Box::new(Node::Alternative{ first, second, }))
        }
        else {
            Ok(first)
        }
    }

    fn parse_toplevel_sequence(input: ParseStream) -> Result<Box<Node>> {
        let subnode = Self::parse_sequence(input)?;
        let action = input.parse::<Block>();
        if action.is_ok() {
            Ok(Box::new(Node::Transformation{ subnode, action: action.unwrap(), }))
        }
        else {
            Ok(subnode)
        }
    }

    fn parse_alternative(input: ParseStream) -> Result<Box<Node>> {
        // Left, pipe and right
        let first = Self::parse_sequence(input)?;
        if input.parse::<Token![|]>().is_ok() {
            let second = Self::parse_alternative(input)?;
            Ok(Box::new(Node::Alternative{ first, second, }))
        }
        else {
            Ok(first)
        }
    }

    fn parse_sequence(input: ParseStream) -> Result<Box<Node>> {
        // Left, pipe and right
        let first = Self::parse_literal(input)?;
        let second = Self::parse_sequence(input);
        if second.is_ok() {
            Ok(Box::new(Node::Sequence{ first, second: second.unwrap(), }))
        }
        else {
            Ok(first)
        }
    }

    fn parse_literal(input: ParseStream) -> Result<Box<Node>> {
        // TODO: Allow parens
        /*if input.parse::<Token![(]>().is_ok() {
            let n = parse_alternative(input)?;
            let _ = input.parse::<Token![)]>()?;
            Ok(n)
        }
        else {*/
            if let Ok(lit) = input.parse::<Lit>() {
                Ok(Box::new(Node::Literal(LiteralNode::Lit(lit))))
            }
            else {
                Ok(Box::new(Node::Literal(LiteralNode::Ident(input.parse()?))))
            }
        //}
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_toplevel_alternative(input).map(|x| *x)
    }
}
