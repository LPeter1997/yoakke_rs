/**
 * The generic interval built on top of the bound representations.
 */

//use std::convert::{From};
use crate::bound::{LowerBound, UpperBound};

/// Represents a generic interval
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Interval<T> {
    pub lower: LowerBound<T>,
    pub upper: UpperBound<T>,
}

/// Represents all the possible relations between two intervals
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IntervalRelation<T> {
    /// No common element
    Disjunct{
        first: Interval<T>,
        second: Interval<T>,
    },

    /// They are touching at one of the interval ends
    Touching{
        first: Interval<T>,
        second: Interval<T>,
    },

    /// They overlap partially
    Overlapping{
        first_disjunct: Interval<T>,
        overlapping: Interval<T>,
        second_disjunct: Interval<T>,
    },

    /// One entirely contains the other
    Containing{
        first_disjunct: Interval<T>,
        overlapping: Interval<T>,
        second_disjunct: Interval<T>,
    },

    /// One interval starts with the other
    Starting{
        overlapping: Interval<T>,
        disjunct: Interval<T>,
    },

    /// One interval finishes with the other
    Finishing{
        disjunct: Interval<T>,
        overlapping: Interval<T>,
    },

    // Completely equal
    Equal(Interval<T>),
}

/**
 * Constructing an interval.
 */

impl <T> Interval<T> {
    pub fn with_bounds(lower: LowerBound<T>, upper: UpperBound<T>) -> Self {
        Self{ lower, upper }
    }

    pub fn full() -> Self {
        Self::with_bounds(LowerBound::Unbounded, UpperBound::Unbounded)
    }
}

impl <T> Interval<T> where T : Clone {
    pub fn singleton(value: T) -> Self {
        Self::with_bounds(LowerBound::Included(value.clone()), UpperBound::Included(value))
    }
}

/**
 * Debug-print an interval.
 */
impl <T> std::fmt::Debug for Interval<T> where T : std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (match &self.lower {
            LowerBound::Excluded(x) => {
                write!(f, "(")?;
                x.fmt(f)
            },
            LowerBound::Included(x) => {
                write!(f, "[")?;
                x.fmt(f)
            },
            LowerBound::Unbounded => {
                write!(f, "(-infty")
            },
        })?;

        write!(f, "; ")?;

        (match &self.upper {
            UpperBound::Excluded(x) => {
                x.fmt(f)?;
                write!(f, ")")
            },
            UpperBound::Included(x) => {
                x.fmt(f)?;
                write!(f, "]")
            },
            UpperBound::Unbounded => {
                write!(f, "+infty)")
            },
        })
    }
}

/**
 * Info about a single interval.
 */

impl <T> Interval<T> {
    pub fn is_full(&self) -> bool {
        self.lower.is_unbounded() && self.upper.is_unbounded()
    }
}

impl <T> Interval<T> where T : PartialOrd {
    /// Checks if an element is contained by the interval.
    pub fn contains(&self, element: &T) -> bool {
        match (&self.lower, &self.upper) {
            (LowerBound::Unbounded, UpperBound::Unbounded) => true,

            (LowerBound::Unbounded, UpperBound::Excluded(x)) => element < x,
            (LowerBound::Unbounded, UpperBound::Included(x)) => element <= x,

            (LowerBound::Excluded(x), UpperBound::Unbounded) => x < element,
            (LowerBound::Included(x), UpperBound::Unbounded) => x <= element,

            (LowerBound::Excluded(x), UpperBound::Excluded(y)) => x < element && element < y,
            (LowerBound::Excluded(x), UpperBound::Included(y)) => x < element && element <= y,
            (LowerBound::Included(x), UpperBound::Excluded(y)) => x <= element && element < y,
            (LowerBound::Included(x), UpperBound::Included(y)) => x <= element && element <= y,
        }
    }

    /// Checks if an interval is empty, meaning that there's no element that's
    /// possibly contained in it.
    pub fn is_empty(&self) -> bool {
        match (&self.lower, &self.upper) {
              (LowerBound::Included(x), UpperBound::Excluded(y))
            | (LowerBound::Excluded(x), UpperBound::Included(y))
            | (LowerBound::Excluded(x), UpperBound::Excluded(y)) => x >= y,

            _ => false,
        }
    }
}

/**
 * Relation of intervals.
 */

impl <T> Interval<T> where T : Ord {
    /// Checks if the first interval is cisjunct with the second one and comes
    /// before it (meaning that it's end is before the second one's start)
    pub fn is_before(&self, other: &Self) -> bool {
        self.upper < other.lower
    }

    /// Checks if two intervals are disjunct
    pub fn is_disjunct(&self, other: &Self) -> bool {
        self.is_before(other) || other.is_before(self)
    }
}

impl <T> Interval<T> where T : Ord + Clone {
    pub fn relates(&self, other: &Self) -> IntervalRelation<T> {
        // We order by the lower bound so we essentially half the number of cases
        let (first, second) = if self.lower < other.lower {
            (self.clone(), other.clone())
        }
        else {
            (other.clone(), self.clone())
        };

        if first.upper.is_touching(&second.lower) {
            IntervalRelation::Touching{ first, second, }
        }
        else if first.upper < second.lower {
            IntervalRelation::Disjunct{ first, second, }
        }
        else if first == second {
            IntervalRelation::Equal(first)
        }
        else if first.lower == second.lower {
            // Starting
            assert!(first.upper != second.upper);
            // Order by upper bound so we only need to handle one case instead of two
            let (first, second) = if first.upper < second.upper {
                (first, second)
            }
            else {
                (second, first)
            };

            IntervalRelation::Starting{
                overlapping: first.clone(),
                disjunct: Interval::with_bounds(first.upper.touching().unwrap(), second.upper)
            }
        }
        else if first.upper == second.upper {
            // Finishing
            assert!(first.lower != second.lower);

            IntervalRelation::Finishing{
                disjunct: Interval::with_bounds(first.lower, second.lower.touching().unwrap()),
                overlapping: second,
            }
        }
        else if first.upper > second.upper {
            // Containing
            IntervalRelation::Containing{
                first_disjunct: Interval::with_bounds(first.lower, second.lower.touching().unwrap()),
                overlapping: second.clone(),
                second_disjunct: Interval::with_bounds(second.upper.touching().unwrap(), first.upper),
            }
        }
        else {
            // Overlapping
            IntervalRelation::Overlapping{
                first_disjunct: Interval::with_bounds(first.lower, second.lower.touching().unwrap()),
                overlapping: Interval::with_bounds(second.lower, first.upper.clone()),
                second_disjunct: Interval::with_bounds(first.upper.touching().unwrap(), second.upper),
            }
        }
    }
}

/**
 * Utilities for interval lists/maps.
 */

// We need this because the standard library doesn't deal with different key types
fn bsearch_by_key<'a, T, K1, K2, F>(
    slice: &'a [T], searched: &'a K1, mut key_selector: F) -> Result<usize, usize>
    where F : FnMut(&'a T) -> &'a K2, K1 : PartialOrd<K2>, K2 : 'a {

    let mut size = slice.len();
    if size == 0 {
        return Err(0);
    }
    let mut base = 0usize;
    while size > 1 {
        let half = size / 2;
        let mid = base + half;
        let key = key_selector(&slice[mid]);
        let cmp = searched.partial_cmp(&key).unwrap();
        base = if cmp == std::cmp::Ordering::Greater { mid } else { base };
        size -= half;
    }

    let key = key_selector(&slice[base]);
    let cmp = searched.partial_cmp(&key).unwrap();
    if cmp == std::cmp::Ordering::Equal {
        Ok(base)
    }
    else {
        Err(base + (cmp == std::cmp::Ordering::Greater) as usize)
    }
}

pub(crate) fn intersecting_index_range<'a, T, K, F>(
    slice: &'a [T], interval: &'a Interval<K>, mut proj: F) -> std::ops::Range<usize>
    where F : FnMut(&'a T) -> &'a Interval<K>, K : Ord {

    let from = match bsearch_by_key(slice, &interval.lower, |x| &proj(x).upper) {
        Ok(idx) | Err(idx) => idx
    };
    let to = match bsearch_by_key(&slice[from..], &interval.upper, |x| &proj(x).lower) {
        Ok(idx) | Err(idx) => idx
    } + from;

    from..to
}

pub(crate) fn touching_index_range<'a, T, K, F>(
    slice: &'a [T], interval: &'a Interval<K>, mut proj: F) -> std::ops::Range<usize>
    where F : FnMut(&'a T) -> &'a Interval<K> + Copy, K : Ord {

    let std::ops::Range{ mut start, mut end, } = intersecting_index_range(slice, interval, proj);

    if start != 0 && interval.lower.is_touching(&proj(&slice[start - 1]).upper) {
        start -= 1;
    }
    if end != slice.len() && interval.upper.is_touching(&proj(&slice[end]).lower) {
        end += 1;
    }

    start..end
}

// Tests ///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod interval_tests {
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

    fn rel<R1, R2, T>(l: R1, r: R2) -> IntervalRelation<T> where T : Clone + Ord,
                    R1 : std::ops::RangeBounds<T>, R2 : std::ops::RangeBounds<T> {
        ri(l).relates(&ri(r))
    }

    fn irs<T>(slice: &[Interval<T>], interval: &Interval<T>) -> (std::ops::Range<usize>, std::ops::Range<usize>)
        where T : Ord {
        (
            intersecting_index_range(slice, interval, |x| x),
            touching_index_range(slice, interval, |x| x)
        )
    }

    /**
     * Relation tests.
     */

    #[test]
    fn a_before_b() {
        assert_eq!(
            rel(1..4, 5..7),
            IntervalRelation::Disjunct{ first: ri(1..4), second: ri(5..7) }
        );
    }

    #[test]
    fn b_before_a() {
        assert_eq!(
            rel(5..7, 1..4),
            IntervalRelation::Disjunct{ first: ri(1..4), second: ri(5..7) }
        );
    }

    #[test]
    fn a_touches_b() {
        assert_eq!(
            rel(1..4, 4..7),
            IntervalRelation::Touching{ first: ri(1..4), second: ri(4..7) }
        );
    }

    #[test]
    fn b_touches_a() {
        assert_eq!(
            rel(4..7, 1..4),
            IntervalRelation::Touching{ first: ri(1..4), second: ri(4..7) }
        );
    }

    #[test]
    fn a_starts_b() {
        assert_eq!(
            rel(4..8, 4..6),
            IntervalRelation::Starting{ overlapping: ri(4..6), disjunct: ri(6..8) }
        );
    }

    #[test]
    fn b_starts_a() {
        assert_eq!(
            rel(4..6, 4..8),
            IntervalRelation::Starting{ overlapping: ri(4..6), disjunct: ri(6..8) }
        );
    }

    #[test]
    fn a_finishes_b() {
        assert_eq!(
            rel(6..8, 4..8),
            IntervalRelation::Finishing{ disjunct: ri(4..6), overlapping: ri(6..8) }
        );
    }

    #[test]
    fn b_finishes_a() {
        assert_eq!(
            rel(4..8, 6..8),
            IntervalRelation::Finishing{ disjunct: ri(4..6), overlapping: ri(6..8) }
        );
    }

    #[test]
    fn a_singleton_intersects_b() {
        assert_eq!(
            rel(4..=6, 6..8),
            IntervalRelation::Overlapping{
                first_disjunct: ri(4..6),
                overlapping: ri(6..=6),
                second_disjunct: Interval::with_bounds(LowerBound::Excluded(6), UpperBound::Excluded(8))
            }
        );
    }

    #[test]
    fn b_singleton_intersects_a() {
        assert_eq!(
            rel(6..8, 4..=6),
            IntervalRelation::Overlapping{
                first_disjunct: ri(4..6),
                overlapping: ri(6..=6),
                second_disjunct: Interval::with_bounds(LowerBound::Excluded(6), UpperBound::Excluded(8))
            }
        );
    }

    #[test]
    fn a_contains_b() {
        assert_eq!(
            rel(2..10, 4..7),
            IntervalRelation::Containing{ first_disjunct: ri(2..4), overlapping: ri(4..7), second_disjunct: ri(7..10) }
        );
    }

    #[test]
    fn b_contains_a() {
        assert_eq!(
            rel(4..7, 2..10),
            IntervalRelation::Containing{ first_disjunct: ri(2..4), overlapping: ri(4..7), second_disjunct: ri(7..10) }
        );
    }

    #[test]
    fn a_intersects_b() {
        assert_eq!(
            rel(2..7, 4..9),
            IntervalRelation::Overlapping{ first_disjunct: ri(2..4), overlapping: ri(4..7), second_disjunct: ri(7..9) }
        );
    }

    #[test]
    fn b_intersects_a() {
        assert_eq!(
            rel(4..9, 2..7),
            IntervalRelation::Overlapping{ first_disjunct: ri(2..4), overlapping: ri(4..7), second_disjunct: ri(7..9) }
        );
    }

    /**
     * The "search" algorithm tests.
     */

    #[test]
    fn search_in_empty() {
        assert_eq!(
            irs(&[], &ri(1..3)),
            (0..0, 0..0)
        );
    }

    #[test]
    fn search_before_all() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(1..2)),
            (0..0, 0..0)
        );
    }

    #[test]
    fn search_after_all() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(25..27)),
            (4..4, 4..4)
        );
    }

    #[test]
    fn search_before_all_touch() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(1..3)),
            (0..0, 0..1)
        );
    }

    #[test]
    fn search_intersect_first() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(1..4)),
            (0..1, 0..1)
        );
    }

    #[test]
    fn search_intersect_first_touch_second() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(1..7)),
            (0..1, 0..2)
        );
    }

    #[test]
    fn search_intersect_first_2() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(1..8)),
            (0..2, 0..2)
        );
    }

    #[test]
    fn search_between() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(10..13)),
            (2..2, 2..2)
        );
    }

    #[test]
    fn search_between_touch_before() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(9..13)),
            (2..2, 1..2)
        );
    }

    #[test]
    fn search_between_touch_after() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(10..14)),
            (2..2, 2..3)
        );
    }

    #[test]
    fn search_between_touch_before_after() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(9..14)),
            (2..2, 1..3)
        );
    }

    #[test]
    fn search_intersects_middle() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(8..15)),
            (1..3, 1..3)
        );
    }

    #[test]
    fn search_intersects_middle_touch_after() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(8..20)),
            (1..3, 1..4)
        );
    }

    #[test]
    fn search_intersects_middle_touch_before() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(6..15)),
            (1..3, 0..3)
        );
    }

    #[test]
    fn search_intersects_middle_touch_before_after() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(6..20)),
            (1..3, 0..4)
        );
    }

    #[test]
    fn search_intersects_extends_beyond_first() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(1..4)),
            (0..1, 0..1)
        );
    }

    #[test]
    fn search_touches_extends_beyond_first() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(1..3)),
            (0..0, 0..1)
        );
    }

    #[test]
    fn search_intersects_extends_beyond_last() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(22..27)),
            (3..4, 3..4)
        );
    }

    #[test]
    fn search_touches_extends_beyond_last() {
        assert_eq!(
            irs(&[ri(3..6), ri(7..9), ri(14..17), ri(20..24)], &ri(24..27)),
            (4..4, 3..4)
        );
    }
}
