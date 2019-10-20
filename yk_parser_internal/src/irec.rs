/**
 * Helper structure for indirect left-recursion.
 */

use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use std::any::Any;
use std::cell::{RefCell, RefMut};
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
    pub head: Option<Rc<RefCell<RecursionHead>>>,
}

impl LeftRecursive {
    pub fn with_parser_and_seed<I, T>(parser: &'static str, seed: ParseResult<I, T>) -> Self where I : 'static, T: 'static {
        Self{ parser, seed: Box::new(seed), head: None }
    }

    pub fn parse_result<I, T>(&self) -> ParseResult<I, T> where I : 'static + Clone, T : 'static + Clone {
        assert!(self.seed.is::<ParseResult<I, T>>());
        (*self.seed.downcast_ref::<ParseResult<I, T>>().unwrap()).clone()
    }
}

// Call head tracking

pub struct CallHeadTable {
    // TODO: This should be more like a weak ref
    heads: HashMap<usize, Rc<RefCell<RecursionHead>>>,
}

impl CallHeadTable {
    pub fn new() -> Self {
        Self{ heads: HashMap::new() }
    }

    pub fn get(&self, idx: &usize) -> Option<&Rc<RefCell<RecursionHead>>> {
        self.heads.get(&idx)
    }

    pub fn insert(&mut self, idx: usize, h: Rc<RefCell<RecursionHead>>) {
        self.heads.insert(idx, h);
    }

    pub fn remove(&mut self, idx: &usize) {
        self.heads.remove(&idx);
    }
}

// Call stack

pub struct CallStack {
    stack: Vec<Rc<RefCell<LeftRecursive>>>,
}

impl CallStack {
    pub fn new() -> Self {
        Self{ stack: Vec::new() }
    }

    pub fn push(&mut self, element: Rc<RefCell<LeftRecursive>>) {
        self.stack.push(element);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn setup_lr(&mut self, parser: &'static str, rec: &Rc<RefCell<LeftRecursive>>) {
        println!("setup_lr_{}()", parser);

        if (*rec).borrow().head.is_none() {
            (*rec).borrow_mut().head = Some(Rc::new(RefCell::new(RecursionHead::with_head(parser))));
        }
        for mut elem in self.stack.iter().rev().map(|x| x.borrow_mut()) {
            if elem.parser == parser {
                break;
            }
            elem.head = (*rec).borrow().head.clone();
            (*rec).borrow().head.as_ref().unwrap().borrow_mut().involved.insert(parser);
        }
    }
}

// Entry in the memo table

#[derive(Clone)]
pub enum Entry<I, T> {
    LeftRecursive(Rc<RefCell<LeftRecursive>>),
    ParseResult(ParseResult<I, T>),
}

impl <I, T> Entry<I, T> where I : Clone, T : Clone {
    pub fn parse_result(&self) -> ParseResult<I, T> where I: 'static, T: 'static {
        match self {
            Entry::LeftRecursive(lr) => (**lr).borrow().parse_result(),
            Entry::ParseResult(r) => r.clone(),
        }
    }
}
