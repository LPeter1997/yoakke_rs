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
 *               | NON_CLOSE_BRACKET
 *               ;
 */

pub fn parse(source: &str) -> Result<Box<Node>, ()> {
    match parse_alternative(source.chars()) {
        Ok((n, _)) => n,
        Err(v) => v,
    }
}

type ParseResult<'a, T> = Result<(T, std::str::Chars<'a>), ()>;

fn parse_alternative(it: std::str::Chars<'_>) -> ParseResult<'_, Box<Node>> {
    unimplemented!();
}

fn parse_sequence(it: std::str::Chars<'_>) -> ParseResult<'_, Box<Node>> {
    unimplemented!();
}

fn parse_quantified(it: std::str::Chars<'_>) -> ParseResult<'_, Box<Node>> {
    unimplemented!();
}

fn parse_quantifier(it: std::str::Chars<'_>) -> ParseResult<'_, Box<Node>> {
    unimplemented!();
}

fn parse_atom(it: std::str::Chars<'_>) -> ParseResult<'_, Box<Node>> {
    unimplemented!();
}

fn parse_grouping(it: std::str::Chars<'_>) -> ParseResult<'_, Box<Node>> {
    unimplemented!();
}

fn parse_grouping_element_init(it: std::str::Chars<'_>) -> ParseResult<'_, GroupingElement> {
    unimplemented!();
}

fn parse_grouping_element(it: std::str::Chars<'_>) -> ParseResult<'_, GroupingElement> {
    unimplemented!();
}

fn parse_grouping_atom_init(it: std::str::Chars<'_>) -> ParseResult<'_, char> {
    unimplemented!();
}

fn parse_grouping_atom(it: std::str::Chars<'_>) -> ParseResult<'_, char> {
    unimplemented!();
}

fn parse_nonspecial_char(it: std::str::Chars<'_>) -> ParseResult<'_, char> {
    unimplemented!();
}

fn parse_special_char(it: std::str::Chars<'_>) -> ParseResult<'_, char> {
    unimplemented!();
}
