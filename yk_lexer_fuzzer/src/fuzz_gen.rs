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
    pub fn with_gen<G>(g: G) -> Self where G : StringGenStrategy {
        Self{ gen: Box::new(g) }
    }
}

impl FuzzStrategy for AppendEdit {
    fn make_edit(&self, src: &str) -> (Range<usize>, String) {
        (src.len()..src.len(), self.gen.generate())
    }
}
