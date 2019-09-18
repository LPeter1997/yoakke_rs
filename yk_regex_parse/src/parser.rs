/**
 * A simple parser to parse a string into a regex AST.
 */

use crate::ast::{Node, Quantifier, GroupingElement};

/*
 * Reference grammar for the parser:
 *
 * alternative ::=
 *               | sequence '|' alternative
 *               | sequence
 *               ;
 *
 * sequence    ::=
 *               | quantified sequence
 *               | quantified
 *               ;
 *
 * quantified  ::=
 *               | atom quantifier
 *               | atom
 *               ;
 *
 * quantifier  ::=
 *               | '?' | '*' | '+'
 *               ;
 *
 * atom        ::=
 *               | '(' alternative ')'
 *               | '[' group ']'
 *               | ANY_NONSPECIAL_CHAR
 *               | '\' ANY_SPECIAL_CHAR
 *               ;
 *
 * group       ::=
 *               | group_init group_rem*
 *               | group_init
 *               ;
 *
 * group_init  ::=
 *               | group_iatom '-' group_atom
 *               | group_iatom
 *               ;
 *
 * group_rem   ::=
 *               | group_atom '-' group_atom
 *               | group_atom
 *               ;
 *
 * group_iatom ::=
 *               | ']'
 *               | group_atom
 *               ;
 *
 * group_atom  ::=
 *               | ANY_NONSPECIAL_CHAR
 *               | '\' ANY_SPECIAL_CHAR
 *               ;
 */

/// A small helper to ease the Chars interface a bit
#[derive(Clone)]
struct Chars<'a>(std::str::Chars<'a>);

impl <'a> Chars<'a> {
    fn next(&self) -> Option<(char, Chars<'a>)> {
        let mut clone = self.clone();
        match clone.0.next() {
            Some(c) => Some((c, clone)),
            None => None,
        }
    }
}

/**
 * Actual parsing.
 */

pub fn parse(source: &str) -> Result<Box<Node>, ()> {
    match parse_alternative(Chars(source.chars())) {
        Ok((n, _)) => Ok(n),
        Err(v) => Err(v),
    }
}

type ParseResult<'a, T> = Result<(T, Chars<'a>), ()>;

fn parse_alternative(it: Chars<'_>) -> ParseResult<'_, Box<Node>> {
    let (first, it) = parse_sequence(it)?;
    if let Some(('|', it)) = it.next() {
        let (second, it) = parse_alternative(it)?;
        Ok((Box::new(Node::Alternative{ first, second }), it))
    }
    else {
        Ok((first, it))
    }
}

fn parse_sequence(it: Chars<'_>) -> ParseResult<'_, Box<Node>> {
    let (first, it) = parse_quantified(it)?;
    if let Ok((second, it)) = parse_sequence(it.clone()) {
        Ok((Box::new(Node::Sequence{ first, second }), it))
    }
    else {
        Ok((first, it))
    }
}

fn parse_quantified(it: Chars<'_>) -> ParseResult<'_, Box<Node>> {
    let (subnode, it) = parse_atom(it)?;
    if let Ok((quantifier, it)) = parse_quantifier(it.clone()) {
        Ok((Box::new(Node::Quantified{ subnode, quantifier }), it))
    }
    else {
        Ok((subnode, it))
    }
}

// TODO: Implement X{N} (At least N) and {N, M} (Between N and M)
fn parse_quantifier(it: Chars<'_>) -> ParseResult<'_, Quantifier> {
    match it.next() {
        Some(('?', it)) => Ok((Quantifier::Between(0, 1), it)),
        Some(('+', it)) => Ok((Quantifier::AtLeast(1), it)),
        Some(('*', it)) => Ok((Quantifier::AtLeast(0), it)),

        _ => Err(()),
    }
}

fn parse_atom(it: Chars<'_>) -> ParseResult<'_, Box<Node>> {
    match it.next() {
        Some(('(', it)) => {
            let (node, it) = parse_alternative(it)?;
            if let Some((')', it)) = it.next() {
                Ok((node, it))
            }
            else {
                Err(())
            }
        },

        Some(('[', it)) => {
            let (node, it) = parse_grouping(it)?;
            if let Some((']', it)) = it.next() {
                Ok((node, it))
            }
            else {
                Err(())
            }
        },

        Some(('\\', it)) => {
            unimplemented!();
        },

        Some((c, it)) => {
            unimplemented!();
        },

        None => Err(()),
    }
}

fn parse_grouping(it: Chars<'_>) -> ParseResult<'_, Box<Node>> {
    unimplemented!();
}

fn parse_grouping_element_init(it: Chars<'_>) -> ParseResult<'_, GroupingElement> {
    unimplemented!();
}

fn parse_grouping_element(it: Chars<'_>) -> ParseResult<'_, GroupingElement> {
    unimplemented!();
}

fn parse_grouping_atom_init(it: Chars<'_>) -> ParseResult<'_, char> {
    unimplemented!();
}

fn parse_grouping_atom(it: Chars<'_>) -> ParseResult<'_, char> {
    unimplemented!();
}

fn parse_nonspecial_char(it: Chars<'_>) -> ParseResult<'_, char> {
    unimplemented!();
}

fn parse_special_char(it: Chars<'_>) -> ParseResult<'_, char> {
    unimplemented!();
}
