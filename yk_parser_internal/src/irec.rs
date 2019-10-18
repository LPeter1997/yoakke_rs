/**
 * Helper structure for indirect left-recursion.
 */

use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use std::any::Any;
use crate::parse_result::ParseResult;

// Recursion head

#[derive(Clone)]
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
    pub parser: String,
    pub seed: Box<dyn Any>,
    pub head: Option<RecursionHead>,
}

impl LeftRecursive {
    pub fn with_parser_and_seed(parser: String, seed: Box<dyn Any>) -> Self {
        Self{ parser, seed, head: None }
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

    pub fn setup_lr(&mut self, parser: &str, rec: &mut LeftRecursive) {
        if rec.head.is_none() {
            rec.head = Some(RecursionHead::with_head(parser.into()));
        }
        for elem in &mut self.stack {
            if elem.parser == parser {
                break;
            }
            Rc::get_mut(elem).unwrap().head = rec.head.clone();
            rec.head.as_mut().unwrap().involved.insert(parser.into());
        }
    }
}

// Entry in the memo table

pub enum Entry<I, T> {
    LeftRecursive(Rc<LeftRecursive>),
    ParseResult(ParseResult<I, T>),
}

impl <I, T> Entry<I, T> where I : Clone, T : Clone {
    pub fn parse_result(&self) -> &ParseResult<I, T> where I: 'static, T: 'static {
        match self {
            Entry::LeftRecursive(lr) =>
                lr.seed.downcast_ref::<ParseResult<I, T>>().unwrap(),
            Entry::ParseResult(r) => r,
        }
    }
}
