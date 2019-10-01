/**
 * Edit generation strategies.
 */

use std::ops::Range;
use crate::rnd::*;
use crate::str_gen::*;

pub trait FuzzStrategy {
    fn make_edit(&self, src: &str) -> (Range<usize>, String);
}

/**
 * Appender, that simply appends a string to the end of the document.
 */

pub struct AppendEdit {
    gen: Box<dyn StringGenStrategy>,
}

impl AppendEdit {
    pub fn with_gen<G>(g: G) -> Self where G : StringGenStrategy + 'static {
        Self{ gen: Box::new(g) }
    }
}

impl FuzzStrategy for AppendEdit {
    fn make_edit(&self, src: &str) -> (Range<usize>, String) {
        (src.len()..src.len(), self.gen.generate())
    }
}

/**
 * Inserter, that inserts a string to a random position into the document.
 */

pub struct InsertEdit {
    gen: Box<dyn StringGenStrategy>,
}

impl InsertEdit {
    pub fn with_gen<G>(g: G) -> Self where G : StringGenStrategy + 'static {
        Self{ gen: Box::new(g) }
    }
}

impl FuzzStrategy for InsertEdit {
    fn make_edit(&self, src: &str) -> (Range<usize>, String) {
        let offs = rand_range(&(0..(src.len() + 1)));
        (offs..offs, self.gen.generate())
    }
}

/**
 * Eraser, that only erases from the document.
 */

pub struct EraseEdit { }

impl EraseEdit {
    pub fn new() -> Self {
        Self{ }
    }
}

impl FuzzStrategy for EraseEdit {
    fn make_edit(&self, src: &str) -> (Range<usize>, String) {
        let min = rand_range(&(0..(src.len() + 1)));
        let max = rand_range(&(min..(src.len() + 1)));
        (min..max, "".into())
    }
}

/**
 * Splicer, that erases and inserts some string.
 */

pub struct SpliceEdit {
    gen: Box<dyn StringGenStrategy>,
}

impl SpliceEdit {
    pub fn with_gen<G>(g: G) -> Self where G : StringGenStrategy + 'static {
        Self{ gen: Box::new(g) }
    }
}

impl FuzzStrategy for SpliceEdit {
    fn make_edit(&self, src: &str) -> (Range<usize>, String) {
        let min = rand_range(&(0..(src.len() + 1)));
        let max = rand_range(&(min..(src.len() + 1)));
        (min..max, self.gen.generate())
    }
}

/**
 * Chooses from a collection of strategies and uses that.
 */

pub struct RandomEdit {
    strats: Vec<Box<dyn FuzzStrategy>>,
}

impl RandomEdit {
    pub fn new() -> Self {
        Self{ strats: Vec::new() }
    }

    pub fn add<G>(&mut self, g: G) where G : FuzzStrategy + 'static {
        self.strats.push(Box::new(g));
    }
}

impl FuzzStrategy for RandomEdit {
    fn make_edit(&self, src: &str) -> (Range<usize>, String) {
        sample(&self.strats).make_edit(src)
    }
}
