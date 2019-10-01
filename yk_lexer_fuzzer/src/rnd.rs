/**
 * Random utilities.
 */

use std::ops::Range;
use rand::Rng;

pub fn rand_range(r: &Range<usize>) -> usize {
    let mut rng = rand::thread_rng();
    rng.gen_range(r.start, r.end)
}

pub fn sample<T>(s: &[T]) -> &T {
    assert!(s.len() > 0);
    let idx = rand_range(&(0..(s.len())));
    &s[idx]
}

pub fn rand_string(len: &Range<usize>, charset: &str) -> String {
    let len = rand_range(len);
    let mut res = String::new();
    for _ in 0..len {
        res.push(*sample(charset.as_bytes()) as char);
    }
    res
}
