/**
 * The generic interval built on top of the bound representations.
 */

//use std::convert::{From};
use crate::bound::{LowerBound, UpperBound};

/// Represents a generic interval
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Interval<T> {
    lower: LowerBound<T>,
    upper: UpperBound<T>,
}

impl <T> Interval<T> {
    pub fn with_bounds(lower: LowerBound<T>, upper: UpperBound<T>) -> Self {
        Self{ lower, upper }
    }
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
 * Info about a single interval.
 */

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

        if first.upper < second.lower {
            IntervalRelation::Disjunct{ first, second, }
        }
        else if first.upper.is_touching(&second.lower) {
            IntervalRelation::Touching{ first, second, }
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
