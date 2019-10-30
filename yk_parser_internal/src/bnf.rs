/**
 * Backus–Naur form AST representation and parsing.
 */

use std::collections::{HashMap, HashSet};
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::{Result, Token, Expr, Block, Ident, Lit, Path, Type};
use syn::parse::{Parse, Parser, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Brace;
use crate::syn_extensions::*;

#[derive(Clone)]
pub struct RuleSet {
    pub item_type: Type,
    pub top_rule: (String, Node),
    pub rules: HashMap<String, (Node, Type)>,
    pub literal_matcher: (),
}

#[derive(Clone)]
pub enum Node {
    Transformation{
        subnode: Box<Node>,
        action: (Brace, TokenStream),
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
    Eps,
}

/**
 * Recursion check.
 */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftRecursion {
    None,
    Direct,
    Indirect,
}

impl RuleSet {
    fn left_recursion_impl(
        n: &Node, rule: &str, rules: &RuleSet, direct: bool, touched: &mut HashSet<String>) -> LeftRecursion {

        match n {
            Node::Alternative{ first, second } => {
                let lr = Self::left_recursion_impl(first, rule, rules, direct, touched);
                let rr = Self::left_recursion_impl(second, rule, rules, direct, touched);
                match (lr, rr) {
                    (LeftRecursion::Indirect, _) | (_, LeftRecursion::Indirect) => LeftRecursion::Indirect,
                    (LeftRecursion::Direct, _) | (_, LeftRecursion::Direct) => LeftRecursion::Direct,
                    _ => LeftRecursion::None,
                }
            },
            Node::Sequence{ first, .. } => Self::left_recursion_impl(first, rule, rules, direct, touched),
            Node::Transformation{ subnode, .. } => Self::left_recursion_impl(subnode, rule, rules, direct, touched),
            Node::Literal(lit) => {
                match lit {
                      LiteralNode::Lit(_)
                    | LiteralNode::Eps => LeftRecursion::None,

                    LiteralNode::Ident(path) => {
                        if path.leading_colon.is_none() && path.segments.len() == 1 {
                            // Simple identifier
                            let ident = path.segments[0].ident.to_string();
                            if ident == rule {
                                if direct {
                                    LeftRecursion::Direct
                                }
                                else {
                                    LeftRecursion::Indirect
                                }
                            }
                            else if touched.contains(&ident) {
                                LeftRecursion::None
                            }
                            else {
                                // Check if it's a rule
                                if let Some((sr, _)) = rules.rules.get(&ident) {
                                    touched.insert(ident);
                                    Self::left_recursion_impl(sr, rule, rules, false, touched)
                                }
                                else {
                                    // Not a rule
                                    LeftRecursion::None
                                }
                            }
                        }
                        else {
                            LeftRecursion::None
                        }
                    }
                }
            }
        }
    }

    pub fn left_recursion(&self, rule: &str) -> LeftRecursion {
        match self.rules.get(rule) {
            Some((n, _)) => Self::left_recursion_impl(n, rule, self, true, &mut HashSet::new()),
            None => LeftRecursion::None,
        }
    }
}

// Parse ///////////////////////////////////////////////////////////////////////

impl Parse for RuleSet {
    fn parse(input: ParseStream) -> Result<Self> {
        //let gname: GrammarName = input.parse()?;
        let itype: ItemType = input.parse()?;

        let decls = input.parse_terminated::<Decl, Token![;]>(Decl::parse)?;

        let mut rules = HashMap::new();
        let mut top_rule = None;
        let mut curr_default_ty = None;

        for decl in decls {
            match decl {
                Decl::RuleType(RuleType{ ty, .. }) => curr_default_ty = Some(ty),

                Decl::Rule(Rule{ ident, ty, node, .. }) => {
                    let ident = ident.to_string();
                    let rty = if let Some(ty) = ty {
                        ty
                    }
                    else if let Some(ty) = curr_default_ty.clone() {
                        ty
                    }
                    else {
                        panic!("No type for rule '{}'!", ident);
                    };

                    if top_rule.is_none() {
                        top_rule = Some((ident.clone(), node.clone()));
                    }
                    rules.insert(ident, (node, rty));
                }
            }
        }

        Ok(Self{
            item_type: itype.ty,
            top_rule: top_rule.unwrap(),
            rules,
            literal_matcher: (),
        })
    }
}

/**
 * item = char;
 */
struct ItemType {
    item_tok: Ident,
    eq: Token![=],
    ty: Type,
    semicol: Token![;],
}

impl Parse for ItemType {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self{
            item_tok: parse_ident(input, "item")?,
            eq: input.parse()?,
            ty: input.parse()?,
            semicol: input.parse()?,
        })
    }
}

/**
 * type = i32
 */

struct RuleType {
    ty_tok: Token![type],
    eq: Token![=],
    ty: Type,
}

impl Parse for RuleType {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self{
            ty_tok: input.parse()?,
            eq: input.parse()?,
            ty: input.parse()?,
        })
    }
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
    ty: Option<Type>,
    col: Token![::],
    eq: Token![=],
    node: Node,
}

impl Parse for Rule {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Rule{
            ident: input.parse()?,
            ty: parse_bracketed_fn(input, |input| input.parse::<Type>()).ok().map(|t| t.1),
            col: input.parse()?,
            eq: input.parse()?,
            node: input.parse()?,
        })
    }
}

/**
 * Since we can have alternating type and rule definitions, we need a sum-type above them.
 */

enum Decl {
    RuleType(RuleType),
    Rule(Rule),
}

impl Parse for Decl {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(ty) = RuleType::parse(input) {
            Ok(Decl::RuleType(ty))
        }
        else {
            Ok(Decl::Rule(Rule::parse(input)?))
        }
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

        let action = Self::parse_transformation(input);
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
        if let Ok(lit) = input.parse::<Lit>() {
            Ok(Box::new(Node::Literal(LiteralNode::Lit(lit))))
        }
        else if let Ok(path) = input.parse::<Path>() {
            Ok(Box::new(Node::Literal(LiteralNode::Ident(path))))
        }
        else if let Ok(_) = input.parse::<Token![$]>() {
            let _ = parse_ident(input, "epsilon")?;
            Ok(Box::new(Node::Literal(LiteralNode::Eps)))
        }
        else {
            let (_, content) = parse_parenthesized_fn(input, Self::parse_alternative)?;
            Ok(content)
        }
    }

    fn parse_transformation(input: ParseStream) -> Result<(Brace, TokenStream)> {
        parse_braced_fn(input, |input| {
            input.step(|cur| {
                let mut ts = TokenStream::new();
                let mut rest = *cur;
                while let Some((tt, next)) = rest.token_tree() {
                    ts.append(tt);
                    rest = next;
                }
                Ok((ts, rest))
            })
        })
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_toplevel_alternative(input).map(|x| *x)
    }
}
