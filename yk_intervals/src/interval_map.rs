/**
 * Associative set from an interval to some value. The intervals are always
 * disjunct but there is a possibility they will touch.
 */

use crate::interval::{Interval, intersecting_index_range};

pub struct IntervalMap<K, V> {
    pub(crate) intervals: Vec<(Interval<K>, V)>,
}

impl <K, V> IntervalMap<K, V> {
    pub fn new() -> Self {
        IntervalMap{ intervals: Vec::new() }
    }
}

pub struct Unification<V> {
    existing: V,
    inserted: V,
}

impl <K, V> IntervalMap<K, V> where K : Ord, V : Clone {
    pub fn insert_and_unify<F>(&mut self, key: Interval<K>, value: V, unify: F)
        where F : FnMut(Unification<V>) -> V {

        if self.intervals.is_empty() {
            self.intervals.push((key, value));
        }
        else {
            let range = intersecting_index_range(&self.intervals, &key, |x| &x.0);
            assert!(range.end >= range.start);

            if range.len() == 0 {
                // Intersects nothing, just insert
                self.intervals.insert(range.start, (key, value));
            }
            else if range.len() == 1 {
                // Intersects with one entry
                // TODO: Implement
            }
            else {
                // Intersects with more entries
                // TODO: Implement
            }
        }
    }
}
