/**
 * Associative set from an interval to some value.
 */

use crate::interval::{Interval};

pub struct IntervalMap<K, V> {
    intervals: Vec<(Interval<K>, V)>,
}

impl <K, V> IntervalMap<K, V> {
    pub fn new() -> Self {
        IntervalMap{ intervals: Vec::new() }
    }
}
