/**
 * String generation strategies.
 */

use std::ops::Range;
use crate::rnd::*;

pub trait StringGenStrategy {
    fn generate(&self) -> String;
}

/**
 * Random string from a charset.
 */

pub struct RandomStringGenerator {
    len: Range<usize>,
    charset: String,
}

impl RandomStringGenerator {
    pub fn with_len_and_charset(len: Range<usize>, charset: &str) -> Self {
        Self{ len, charset: charset.into() }
    }
}

impl StringGenStrategy for RandomStringGenerator {
    fn generate(&self) -> String {
        rand_string(&self.len, &self.charset)
    }
}

/**
 * Random string from a predefined set of strings.
 */

pub struct RandomTokenGenerator {
    strs: Vec<String>,
}

impl RandomTokenGenerator {
    pub fn new() -> Self {
        Self{ strs: Vec::new() }
    }

    pub fn add(&mut self, s: &str) {
        self.strs.push(s.into());
    }
}

impl StringGenStrategy for RandomTokenGenerator {
    fn generate(&self) -> String {
        sample(&self.strs).clone()
    }
}

/**
 * Chooses from a collection of strategies and uses that.
 */

pub struct RandomStringStrategy {
    strats: Vec<Box<dyn StringGenStrategy>>,
}

impl RandomStringStrategy {
    pub fn new() -> Self {
        Self{ strats: Vec::new() }
    }

    pub fn add<G>(&mut self, g: G) where G : StringGenStrategy + 'static {
        self.strats.push(Box::new(g));
    }
}

impl StringGenStrategy for RandomStringStrategy {
    fn generate(&self) -> String {
        sample(&self.strats).generate()
    }
}
