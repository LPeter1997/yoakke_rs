/**
 * Associative set from an interval to some value. The intervals are always
 * disjunct but there is a possibility they will touch.
 */

use crate::interval::{Interval, IntervalRelation, intersecting_index_range};

#[derive(Debug, PartialEq, Eq)]
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

impl <K, V> IntervalMap<K, V> where K : Clone + Ord, V : Clone {
    // TODO: Check for extra clone()s here
    // TODO: For multiple insertions we could use splice
    fn insert_and_unify_single<F>(
        &mut self, idx: usize, key: Interval<K>, value: V, mut unify: F)
        where F : FnMut(Unification<V>) -> V {

        let mut unify_fn = |existing, inserted| {
            unify(Unification{ existing, inserted, })
        };

        let existing = &mut self.intervals[idx];
        let rel = existing.0.relates(&key);

        match rel {
            IntervalRelation::Equal(_) => {
                existing.1 = unify_fn(existing.1.clone(), value);
            },

            IntervalRelation::Containing{ first_disjunct, overlapping, second_disjunct } => {
                if existing.0.lower == overlapping.lower {
                    // The newly added interval completely covers the existing one
                    assert!(existing.0 == overlapping);
                    assert!(key.lower == first_disjunct.lower);
                    assert!(key.upper == second_disjunct.upper);

                    // Unify to the existing
                    existing.1 = unify_fn(existing.1.clone(), value.clone());
                    // Add the sides (first the latter one so the indicies don't shift)
                    self.intervals.insert(idx + 1, (second_disjunct, value.clone()));
                    self.intervals.insert(idx, (first_disjunct, value));
                }
                else {
                    // The newly added interval is completely inside the existing one
                    assert!(key == overlapping);
                    assert!(existing.0.lower == first_disjunct.lower);
                    assert!(existing.0.upper == second_disjunct.upper);

                    // We need to do some splitting in this case
                    // We modify the existing interval to be the lower disjunct one
                    existing.0.upper = first_disjunct.upper;
                    // Then we add 2 entries after it
                    let ex_clone = existing.1.clone();
                    self.intervals.insert(idx + 1, (overlapping, unify_fn(ex_clone.clone(), value.clone())));
                    self.intervals.insert(idx + 2, (second_disjunct, ex_clone));
                }
            },

            IntervalRelation::Overlapping{ first_disjunct, overlapping, second_disjunct } => {
                if existing.0.lower == first_disjunct.lower {
                    assert!(existing.0.upper == overlapping.upper);
                    assert!(key.lower == overlapping.lower);
                    assert!(key.upper == second_disjunct.upper);

                    // Resize the existing interval to be the lower disjunct
                    existing.0.upper = first_disjunct.upper;
                    // Add two extra intervals after that
                    let ex_clone = existing.1.clone();
                    self.intervals.insert(idx + 1, (overlapping, unify_fn(ex_clone.clone(), value.clone())));
                    self.intervals.insert(idx + 2, (second_disjunct, value));
                }
                else {
                    assert!(key.lower == first_disjunct.lower);
                    assert!(key.upper == overlapping.upper);
                    assert!(existing.0.lower == overlapping.lower);
                    assert!(existing.0.upper == second_disjunct.upper);

                    // Resize the existing interval to be the upper disjunct
                    existing.0.lower = second_disjunct.lower;
                    // Add two extra intervals before that
                    let ex_clone = existing.1.clone();
                    self.intervals.insert(idx, (overlapping, unify_fn(ex_clone, value.clone())));
                    self.intervals.insert(idx, (first_disjunct, value));
                }
            },

            IntervalRelation::Starting{ overlapping, disjunct } => {
                assert!(existing.0.lower == overlapping.lower);
                assert!(key.lower == overlapping.lower);

                if existing.0.upper == overlapping.upper {
                    assert!(existing.0 == overlapping);
                    assert!(key.upper == disjunct.upper);

                    existing.1 = unify_fn(existing.1.clone(), value.clone());
                    // Add an extra entry after
                    self.intervals.insert(idx + 1, (disjunct, value));
                }
                else {
                    assert!(key == overlapping);
                    assert!(existing.0.upper == disjunct.upper);

                    let ex_clone = existing.1.clone();
                    existing.1 = unify_fn(ex_clone.clone(), value.clone());
                    // Add an extra entry after
                    self.intervals.insert(idx + 1, (disjunct, ex_clone));
                }
            },

            IntervalRelation::Finishing{ disjunct, overlapping } => {
                assert!(existing.0.upper == overlapping.upper);
                assert!(key.upper == overlapping.upper);

                if existing.0.lower == disjunct.lower {
                    assert!(key == overlapping);

                    let ex_clone = existing.1.clone();
                    existing.1 = unify_fn(ex_clone.clone(), value);
                    // Extra entry before
                    self.intervals.insert(idx, (disjunct, ex_clone));
                }
                else {
                    assert!(existing.0 == overlapping);

                    let ex_clone = existing.1.clone();
                    existing.1 = unify_fn(ex_clone.clone(), value.clone());
                    // Extra entry before
                    self.intervals.insert(idx, (disjunct, value));
                }
            },

              IntervalRelation::Touching{ .. }
            | IntervalRelation::Disjunct{ .. } => panic!("Impossible!"),
        }
    }

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

                self.insert_and_unify_single(range.start, key, value, unify);
            }
            else {
                // Intersects with more entries
                // TODO: Implement
                unimplemented!();
            }
        }
    }
}

// Tests ///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod interval_map_tests {
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

    // Helper unify method
    fn test_unify<T>(mut u: Unification<Vec<T>>) -> Vec<T> {
        u.existing.extend(u.inserted);
        u.existing
    }

    macro_rules! ivmap {
        ( $( $x:expr => $y:expr ),* ) => {
            {
                let mut iv = IntervalMap::new();
                $(
                    iv.insert_and_unify(ri($x), $y, test_unify);
                )*
                iv
            }
        };
    }

    macro_rules! ivmap_raw {
        ( $( $x:expr => $y:expr ),* ) => {
            {
                IntervalMap{
                    intervals: vec![
                        $( (ri($x), $y) ),*
                    ],
                }
            }
        };
    }

    /**
     * Insert and unify tests.
     */

    #[test]
    fn insert_into_empty_map() {
        let mut map = IntervalMap::new();
        assert_eq!(map.intervals, vec![]);
        map.insert_and_unify(ri(2..3), vec![1], test_unify);
        assert_eq!(map.intervals, vec![(ri(2..3), vec![1])]);
        assert_eq!(map, ivmap![2..3 => vec![1]]);
    }

    #[test]
    fn insert_into_map_disjunct_before() {
        let mut map = ivmap_raw![5..7 => vec![1], 12..15 => vec![1]];
        assert_eq!(map.intervals, vec![(ri(5..7), vec![1]), (ri(12..15), vec![1])]);
        map.insert_and_unify(ri(2..3), vec![2], test_unify);
        assert_eq!(map.intervals, vec![(ri(2..3), vec![2]), (ri(5..7), vec![1]), (ri(12..15), vec![1])]);
        assert_eq!(map, ivmap![2..3 => vec![2], 5..7 => vec![1], 12..15 => vec![1]]);
    }

    #[test]
    fn insert_into_map_disjunct_before_touch() {
        let mut map = ivmap_raw![5..7 => vec![1], 12..15 => vec![1]];
        assert_eq!(map.intervals, vec![(ri(5..7), vec![1]), (ri(12..15), vec![1])]);
        map.insert_and_unify(ri(2..5), vec![2], test_unify);
        assert_eq!(map.intervals, vec![(ri(2..5), vec![2]), (ri(5..7), vec![1]), (ri(12..15), vec![1])]);
        assert_eq!(map, ivmap![2..5 => vec![2], 5..7 => vec![1], 12..15 => vec![1]]);
    }

    #[test]
    fn insert_into_map_disjunct_after() {
        let mut map = ivmap_raw![5..7 => vec![1], 12..15 => vec![1]];
        map.insert_and_unify(ri(17..19), vec![2], test_unify);
        assert_eq!(map, ivmap![5..7 => vec![1], 12..15 => vec![1], 17..19 => vec![2]]);
    }

    #[test]
    fn insert_into_map_disjunct_after_touch() {
        let mut map = ivmap_raw![5..7 => vec![1], 12..15 => vec![1]];
        map.insert_and_unify(ri(15..19), vec![2], test_unify);
        assert_eq!(map, ivmap![5..7 => vec![1], 12..15 => vec![1], 15..19 => vec![2]]);
    }

    #[test]
    fn insert_into_map_single_equal() {
        let mut map = ivmap_raw![5..7 => vec![1]];
        map.insert_and_unify(ri(5..7), vec![2], test_unify);
        assert_eq!(map, ivmap![5..7 => vec![1, 2]]);
    }

    #[test]
    fn insert_into_map_single_containing_inserted() {
        let mut map = ivmap_raw![3..9 => vec![1]];
        map.insert_and_unify(ri(5..7), vec![2], test_unify);
        assert_eq!(map, ivmap![3..5 => vec![1], 5..7 => vec![1, 2], 7..9 => vec![1]]);
    }

    #[test]
    fn insert_into_map_single_containing_existing() {
        let mut map = ivmap_raw![5..7 => vec![1]];
        map.insert_and_unify(ri(3..9), vec![2], test_unify);
        assert_eq!(map, ivmap![3..5 => vec![2], 5..7 => vec![1, 2], 7..9 => vec![2]]);
    }

    #[test]
    fn insert_into_map_single_overlapping_left() {
        let mut map = ivmap_raw![5..9 => vec![1]];
        map.insert_and_unify(ri(2..7), vec![2], test_unify);
        assert_eq!(map, ivmap![2..5 => vec![2], 5..7 => vec![1, 2], 7..9 => vec![1]]);
    }

    #[test]
    fn insert_into_map_single_overlapping_right() {
        let mut map = ivmap_raw![5..9 => vec![1]];
        map.insert_and_unify(ri(7..12), vec![2], test_unify);
        assert_eq!(map, ivmap![5..7 => vec![1], 7..9 => vec![1, 2], 9..12 => vec![2]]);
    }
}
