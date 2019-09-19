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

// TODO: There could be a lit cleaned up VIA pattern guards, but the stabilization
// of bind-by-move is two versions away:
// https://github.com/rust-lang/rust/pull/63118

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
            if is_nonspecial_char(c) {
                Ok((Box::new(Node::Literal(c)), it))
            }
            else {
                Err(())
            }
        },

        None => Err(()),
    }
}

fn parse_grouping(it: Chars<'_>) -> ParseResult<'_, Box<Node>> {
    // TODO: Negated value
    let (first, mut it) = parse_grouping_element_init(it)?;
    let mut elements = vec![first];
    while let Ok((nth, nextit)) = parse_grouping_element(it.clone()) {
        it = nextit;
        elements.push(nth);
    }
    Ok((Box::new(Node::Grouping{ negated: false, elements }), it))
}

fn parse_grouping_element_init(it: Chars<'_>) -> ParseResult<'_, GroupingElement> {
    let (left, it) = parse_grouping_atom_init(it)?;
    if let Some(('-', it)) = it.next() {
        if let Ok((right, it)) = parse_grouping_atom(it) {
            return Ok((GroupingElement::Range(left, right), it));
        }
    }
    return Ok((GroupingElement::Literal(left), it));
}

fn parse_grouping_element(it: Chars<'_>) -> ParseResult<'_, GroupingElement> {
    let (left, it) = parse_grouping_atom(it)?;
    if let Some(('-', it)) = it.next() {
        if let Ok((right, it)) = parse_grouping_atom(it) {
            return Ok((GroupingElement::Range(left, right), it));
        }
    }
    return Ok((GroupingElement::Literal(left), it));
}

fn parse_grouping_atom_init(it: Chars<'_>) -> ParseResult<'_, char> {
    if let Some((']', it)) = it.next() {
        Ok((']', it))
    }
    else {
        parse_grouping_atom(it)
    }
}

fn parse_grouping_atom(it: Chars<'_>) -> ParseResult<'_, char> {
    match it.next() {
        Some((']', _)) => Err(()),

        Some(('\\', it)) => {
            unimplemented!();
        },

        Some((c, it)) => {
            if is_nonspecial_char(c) {
                Ok((c, it))
            }
            else {
                Err(())
            }
        },

        None => Err(()),
    }
}

fn is_nonspecial_char(c: char) -> bool {
    !c.is_control() && !is_special_char(c)
}

fn is_special_char(c: char) -> bool {
    "()[]?*+|".contains(c)
}

// Tests ///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod regex_parser_tests {
    use super::*;

    /**
     * Helpers to construct results.
     */

    fn alt(first: Box<Node>, second: Box<Node>) -> Box<Node> {
        Box::new(Node::Alternative{ first, second })
    }

    fn seq(first: Box<Node>, second: Box<Node>) -> Box<Node> {
        Box::new(Node::Sequence{ first, second })
    }

    fn grp(negated: bool, elements: Vec<GroupingElement>) -> Box<Node> {
        Box::new(Node::Grouping{ negated, elements })
    }

    fn ch(c: char) -> Box<Node> {
        Box::new(Node::Literal(c))
    }

    fn ge_ch(c: char) -> GroupingElement {
        GroupingElement::Literal(c)
    }

    fn ge_rng(a: char, b: char) -> GroupingElement {
        GroupingElement::Range(a, b)
    }

    fn star(subnode: Box<Node>) -> Box<Node> {
        Box::new(Node::Quantified{ subnode, quantifier: Quantifier::AtLeast(0) })
    }

    fn plus(subnode: Box<Node>) -> Box<Node> {
        Box::new(Node::Quantified{ subnode, quantifier: Quantifier::AtLeast(1) })
    }

    fn qmark(subnode: Box<Node>) -> Box<Node> {
        Box::new(Node::Quantified{ subnode, quantifier: Quantifier::Between(0, 1) })
    }

    /**
     * Actual tests.
     */

    #[test]
    fn a_or_b() {
        assert_eq!(parse(r"a|b"), Ok(alt(ch('a'), ch('b'))));
    }

    #[test]
    fn a_or_b_or_c() {
        assert_eq!(parse(r"a|b|c"), Ok(alt(ch('a'), alt(ch('b'), ch('c')))));
    }

    #[test]
    fn ab() {
        assert_eq!(parse(r"ab"), Ok(seq(ch('a'), ch('b'))));
    }

    #[test]
    fn abc() {
        assert_eq!(parse(r"abc"), Ok(seq(ch('a'), seq(ch('b'), ch('c')))));
    }

    #[test]
    fn ab_or_c() {
        assert_eq!(parse(r"ab|c"), Ok(alt(seq(ch('a'), ch('b')), ch('c'))));
    }

    #[test]
    fn ab_or_cd() {
        assert_eq!(parse(r"ab|cd"), Ok(alt(seq(ch('a'), ch('b')), seq(ch('c'), ch('d')))));
    }

    #[test]
    fn a_b_or_c_d() {
        assert_eq!(parse(r"a(b|c)d"), Ok(seq(ch('a'), seq(alt(ch('b'), ch('c')), ch('d')))));
    }

    #[test]
    fn abc_group() {
        assert_eq!(parse(r"[abc]"), Ok(grp(false, vec![ge_ch('a'), ge_ch('b'), ge_ch('c')])));
    }

    #[test]
    fn a_to_c_and_f_to_h_group() {
        assert_eq!(parse(r"[a-cf-h]"), Ok(grp(false, vec![ge_rng('a', 'c'), ge_rng('f', 'h')])));
    }

    #[test]
    fn a_zero_or_more() {
        assert_eq!(parse(r"a*"), Ok(star(ch('a'))));
    }

    #[test]
    fn a_then_b_zero_or_more() {
        assert_eq!(parse(r"ab*"), Ok(seq(ch('a'), star(ch('b')))));
    }

    #[test]
    fn ab_zero_or_more() {
        assert_eq!(parse(r"(ab)*"), Ok(star(seq(ch('a'), ch('b')))));
    }
}
