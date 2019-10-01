/**
 * Random utilities.
 */

use std::cell::RefCell;
use std::ops::Range;
use std::time::SystemTime;
use rand::{Rng, SeedableRng};
use rand_pcg::Mcg128Xsl64;

thread_local! {
    static MY_RNG: RefCell<Mcg128Xsl64> = RefCell::new(Mcg128Xsl64::new(0));
    static CURR_SEED: RefCell<u64> = RefCell::new(0);
}

pub fn seed_from_system_time() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}

pub fn current_seed() -> u64 {
    CURR_SEED.with(|s| {
        *s.borrow()
    })
}

pub fn set_seed(s: u64) {
    MY_RNG.with(|rng| {
        *rng.borrow_mut() = Mcg128Xsl64::seed_from_u64(s);
    });
    CURR_SEED.with(|se| {
        *se.borrow_mut() = s;
    });
}

pub fn rand_range(r: &Range<usize>) -> usize {
    MY_RNG.with(|rng| {
        rng.borrow_mut().gen_range(r.start, r.end)
    })
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
