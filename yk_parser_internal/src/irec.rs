/**
 * Helper structure for indirect left-recursion.
 */

use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use crate::parse_result::ParseResult;

// Recursion head

pub struct RecursionHead {
    pub head: String,
    pub involved: HashSet<String>,
    pub eval: HashSet<String>,
}

impl RecursionHead {
    pub fn with_head(head: String) -> Self {
        Self{ head, involved: HashSet::new(), eval: HashSet::new() }
    }
}

// Left recursive

pub struct LeftRecursive {
    // TODO: Left out seed, didn't see usage anywhere
    pub parser: String,
    pub head: Option<RecursionHead>,
}

impl LeftRecursive {
    pub fn with_parser(parser: String) -> Self {
        Self{ parser, head: None }
    }
}

// Call head tracking

pub struct CallHeadTable<'a> {
    heads: HashMap<usize, &'a RecursionHead>,
}

impl <'a> CallHeadTable<'a> {
    pub fn new() -> Self {
        Self{ heads: HashMap::new() }
    }

    pub fn get(&self, idx: usize) -> Option<&'a RecursionHead> {
        self.heads.get(&idx).map(|h| *h)
    }

    pub fn remove(&mut self, idx: usize) {
        self.heads.remove(&idx);
    }
}

// Call stack

pub struct CallStack {
    stack: Vec<Rc<LeftRecursive>>,
}

impl CallStack {
    pub fn new() -> Self {
        Self{ stack: Vec::new() }
    }

    pub fn push(&mut self, element: Rc<LeftRecursive>) {
        self.push(element);
    }

    pub fn pop(&mut self) {
        self.pop();
    }
}
