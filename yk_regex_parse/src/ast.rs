/**
 * All of the regex syntax-tree (AST) data-structures.
 */

// TODO: Character classes, like [[:alpha:]], ...

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

    Quantified{
        subnode: Box<Node>,
        quantifier: Quantifier,
    },

    Grouping{
        negated: bool,
        elements: Vec<GroupingElement>,
    },

    Literal(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantifier {
    AtLeast(usize),
    Between(usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupingElement {
    Literal(char),
    Range(char, char),
}
