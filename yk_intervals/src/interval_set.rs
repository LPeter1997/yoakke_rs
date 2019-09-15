/**
 * Stores a set of disjunct intervals, unifying them when possible.
 */

use crate::interval::{Interval, touching_index_range};

pub struct IntervalSet<T> {
    pub(crate) intervals: Vec<Interval<T>>,
}

impl <T> IntervalSet<T> {
    pub fn new() -> Self {
        IntervalSet{ intervals: Vec::new() }
    }
}

impl <T> IntervalSet<T> where T : Ord {
    // TODO: Does it make sense for this to return anything?
    /// Inserts an interval into the set, unifying every touching and
    /// overlapping entry.
    pub fn insert(&mut self, value: Interval<T>) {
        if self.intervals.is_empty() {
            self.intervals.push(value);
        }
        else {
            let range = touching_index_range(&self.intervals, &value, |x| x);
            assert!(range.end >= range.start);

            if range.len() == 0 {
                // Intersects or touches nothing, just insert
                self.intervals.insert(range.start, value);
            }
            else if range.len() == 1 {
                // Intersects with a single entry, unify
                let idx = range.start;

                self.intervals[idx].lower = std::cmp::min(self.intervals[idx].lower, value.lower);
                self.intervals[idx].upper = std::cmp::max(self.intervals[idx].upper, value.upper);
            }
            else {
                // Intersects with multiple entries, we need to unify and remove
                /*
                let slice = &mut self.intervals[range];

                let first = slice.first_mut().unwrap();
                let last = slice.last().unwrap();

                // Unify
                first.lower = std::cmp::min(first.lower, value.lower);
                first.upper = std::cmp::max(last.upper, value.upper);

                // Remove the rest
                self.intervals.drain((range.start + 1)..range.end);
                */
            }
        }
    }
}
