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
    pub head: &'static str,
    pub involved: HashSet<&'static str>,
    pub eval: HashSet<&'static str>,
}

impl RecursionHead {
    pub fn with_head(head: &'static str) -> Self {
        Self{ head, involved: HashSet::new(), eval: HashSet::new() }
    }
}

// Left recursive

pub struct LeftRecursive {
    pub parser: &'static str,
    pub seed: Box<dyn Any>,
    pub head: Option<RecursionHead>,
}

impl LeftRecursive {
    pub fn with_parser_and_seed<I, T>(parser: &'static str, seed: ParseResult<I, T>) -> Self where I : 'static, T: 'static {
        Self{ parser, seed: Box::new(seed), head: None }
    }

    pub fn parse_result<I, T>(&self) -> Option<&ParseResult<I, T>> where I : 'static, T : 'static {
        self.seed.downcast_ref::<ParseResult<I, T>>()
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

    pub fn setup_lr(&mut self, parser: &'static str, rec: &mut LeftRecursive) {
        if rec.head.is_none() {
            rec.head = Some(RecursionHead::with_head(parser));
        }
        for elem in &mut self.stack.iter_mut().rev() {
            if elem.parser == parser {
                break;
            }
            Rc::get_mut(elem).unwrap().head = rec.head.clone();
            rec.head.as_mut().unwrap().involved.insert(parser);
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
            Entry::LeftRecursive(lr) => lr.parse_result().unwrap(),
            Entry::ParseResult(r) => r,
        }
    }
}
