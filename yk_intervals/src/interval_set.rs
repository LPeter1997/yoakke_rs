/**
 * Stores a set of disjunct intervals, unifying them when possible.
 */

use crate::interval::{Interval, touching_index_range};

#[derive(Debug, PartialEq, Eq)]
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
                // Intersects with a single entry
                let idx = range.start;

                // Unify that single interval
                if value.lower < self.intervals[idx].lower {
                    self.intervals[idx].lower = value.lower;
                }
                if value.upper > self.intervals[idx].upper {
                    self.intervals[idx].upper = value.upper;
                }
            }
            else {
                // Intersects with multiple entries, we need to unify and remove
                let fst_idx = range.start;

                // Unify into the first interval
                if value.lower < self.intervals[fst_idx].lower {
                    self.intervals[fst_idx].lower = value.lower;
                }

                // Since we are moving the upper, we need to drain here all the unnecessary elements
                // and take the last one for unification
                let last = self.intervals.drain((fst_idx + 1)..range.end).rev().next().unwrap();
                if value.upper > last.upper {
                    self.intervals[fst_idx].upper = value.upper;
                }
                else {
                    // Since it's another entry, we need to copy that upper value
                    self.intervals[fst_idx].upper = last.upper;
                }
            }
        }
    }
}

// Tests ///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod interval_set_tests {
    use super::*;

    fn bound_clone<T>(b: std::ops::Bound<&T>) -> std::ops::Bound<T> where T : Clone {
        match b {
            std::ops::Bound::Excluded(x) => std::ops::Bound::Excluded(x.clone()),
            std::ops::Bound::Included(x) => std::ops::Bound::Included(x.clone()),
            std::ops::Bound::Unbounded => std::ops::Bound::Unbounded,
        }
    }

    fn range_to_interval<R, T>(r: R) -> Interval<T> where R : std::ops::RangeBounds<T>, T : Clone {
        Interval::with_bounds(bound_clone(r.start_bound()).into(), bound_clone(r.end_bound()).into())
    }

    // Just to make it easier to type
    fn ri<R, T>(r: R) -> Interval<T> where R : std::ops::RangeBounds<T>, T : Clone {
        range_to_interval(r)
    }

    macro_rules! ivset {
        ( $( $x:expr ),* ) => {
            {
                let mut iv = IntervalSet::new();
                $(
                    iv.insert(ri($x));
                )*
                iv
            }
        };
    }

    macro_rules! ivset_raw {
        ( $( $x:expr ),* ) => {
            {
                IntervalSet{
                    intervals: vec![
                        $( ri($x) ),*
                    ],
                }
            }
        };
    }

    #[test]
    fn insert_into_empty_set() {
        let mut set = IntervalSet::new();
        assert_eq!(set.intervals, vec![]);
        set.insert(ri(2..3));
        assert_eq!(set.intervals, vec![ri(2..3)]);
        assert_eq!(set, ivset![2..3]);
    }

    #[test]
    fn insert_into_set_dijsunct_before() {
        let mut set = ivset_raw![5..7, 12..15];
        assert_eq!(set.intervals, vec![ri(5..7), ri(12..15)]);
        set.insert(ri(2..3));
        assert_eq!(set.intervals, vec![ri(2..3), ri(5..7), ri(12..15)]);
        assert_eq!(set, ivset![2..3, 5..7, 12..15]);
    }

    #[test]
    fn insert_into_set_dijsunct_after() {
        let mut set = ivset_raw![5..7, 12..15];
        assert_eq!(set.intervals, vec![ri(5..7), ri(12..15)]);
        set.insert(ri(2..3));
        assert_eq!(set.intervals, vec![ri(2..3), ri(5..7), ri(12..15)]);
        assert_eq!(set, ivset![2..3, 5..7, 12..15]);
    }

    #[test]
    fn insert_into_set_dijsunct_between() {
        let mut set = ivset_raw![5..7, 12..15];
        set.insert(ri(8..11));
        assert_eq!(set, ivset![5..7, 8..11, 12..15]);
    }

    #[test]
    fn insert_into_set_touch_first() {
        let mut set = ivset_raw![5..7, 12..15];
        set.insert(ri(3..5));
        assert_eq!(set, ivset![3..7, 12..15]);
    }

    #[test]
    fn insert_into_set_intersect_first() {
        let mut set = ivset_raw![5..7, 12..15];
        set.insert(ri(3..6));
        assert_eq!(set, ivset![3..7, 12..15]);
    }

    #[test]
    fn insert_into_set_touch_between() {
        let mut set = ivset_raw![5..7, 12..15];
        set.insert(ri(7..12));
        assert_eq!(set, ivset![5..15]);
    }

    #[test]
    fn insert_into_set_overextend_all() {
        let mut set = ivset_raw![5..7, 12..15];
        set.insert(ri(3..16));
        assert_eq!(set, ivset![3..16]);
    }
}
