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
