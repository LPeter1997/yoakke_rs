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
    // TODO: This should be more like an owning ref
    pub head: Option<Rc<RecursionHead>>,
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

pub struct CallHeadTable {
    // TODO: This should be more like a weak ref
    heads: HashMap<usize, Rc<RecursionHead>>,
}

impl CallHeadTable {
    pub fn new() -> Self {
        Self{ heads: HashMap::new() }
    }

    pub fn get(&self, idx: &usize) -> Option<&Rc<RecursionHead>> {
        self.heads.get(&idx)
    }

    pub fn get_mut(&mut self, idx: &usize) -> Option<&mut Rc<RecursionHead>> {
        self.heads.get_mut(&idx)
    }

    pub fn remove(&mut self, idx: &usize) {
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
        self.stack.push(element);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn setup_lr(&mut self, parser: &'static str, rec: &mut LeftRecursive) {
        if rec.head.is_none() {
            rec.head = Some(Rc::new(RecursionHead::with_head(parser)));
        }
        for elem in &mut self.stack.iter_mut().rev() {
            if elem.parser == parser {
                break;
            }
            Rc::get_mut(elem).unwrap().head = rec.head.clone();
            Rc::get_mut(rec.head.as_mut().unwrap()).unwrap().involved.insert(parser);
        }
    }
}

// Entry in the memo table

#[derive(Clone)]
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
