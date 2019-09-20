/**
 * Here is a generic lower and upper bound representation for intervals.
 */

use std::cmp::Ordering;

/// Represents the lower bound of an interval
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LowerBound<T> {
    Unbounded,
    Excluded(T),
    Included(T),
}

/// Represents the upper bound of an interval
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpperBound<T> {
    Unbounded,
    Excluded(T),
    Included(T),
}

/**
 * Some observers.
 */

impl <T> LowerBound<T> {
    pub fn is_unbounded(&self) -> bool {
        match self {
            LowerBound::Unbounded => true,
            _ => false,
        }
    }
}

impl <T> UpperBound<T> {
    pub fn is_unbounded(&self) -> bool {
        match self {
            UpperBound::Unbounded => true,
            _ => false,
        }
    }
}

/**
 * Total ordering for lower-bound.
 */

impl <T> PartialOrd for LowerBound<T> where T : Ord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl <T> Ord for LowerBound<T> where T : Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (LowerBound::Unbounded, LowerBound::Unbounded) => Ordering::Equal,
            (LowerBound::Unbounded, _) => Ordering::Less,
            (_, LowerBound::Unbounded) => Ordering::Greater,

              (LowerBound::Excluded(x), LowerBound::Excluded(y))
            | (LowerBound::Included(x), LowerBound::Included(y)) => x.cmp(y),

            (LowerBound::Included(x), LowerBound::Excluded(y)) =>
                match x.cmp(y) {
                    Ordering::Equal => Ordering::Less,
                    o => o,
                },

            (LowerBound::Excluded(x), LowerBound::Included(y)) =>
                match x.cmp(y) {
                    Ordering::Equal => Ordering::Greater,
                    o => o,
                },
        }
    }
}

/**
 * Total ordering for upper-bound.
 */

impl <T> PartialOrd for UpperBound<T> where T : Ord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl <T> Ord for UpperBound<T> where T : Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (UpperBound::Unbounded, UpperBound::Unbounded) => Ordering::Equal,
            (UpperBound::Unbounded, _) => Ordering::Greater,
            (_, UpperBound::Unbounded) => Ordering::Less,

              (UpperBound::Excluded(x), UpperBound::Excluded(y))
            | (UpperBound::Included(x), UpperBound::Included(y)) => x.cmp(y),

            (UpperBound::Included(x), UpperBound::Excluded(y)) =>
                match x.cmp(y) {
                    Ordering::Equal => Ordering::Greater,
                    o => o,
                },

            (UpperBound::Excluded(x), UpperBound::Included(y)) =>
                match x.cmp(y) {
                    Ordering::Equal => Ordering::Less,
                    o => o,
                },
        }
    }
}

/**
 * Partial equality between lower and upper bounds.
 */

impl <T> PartialEq<UpperBound<T>> for LowerBound<T> where T : Eq {
    fn eq(&self, _: &UpperBound<T>) -> bool {
        false
    }
}

impl <T> PartialEq<LowerBound<T>> for UpperBound<T> where T : Eq {
    fn eq(&self, other: &LowerBound<T>) -> bool {
        other.eq(self)
    }
}

/**
 * Partial ordering between lower and upper bounds.
 */

impl <T> PartialOrd<UpperBound<T>> for LowerBound<T> where T : Ord {
    fn partial_cmp(&self, other: &UpperBound<T>) -> Option<Ordering> {
        match (self, other) {
              (LowerBound::Unbounded, UpperBound::Unbounded)
            | (LowerBound::Unbounded, _)
            | (_, UpperBound::Unbounded) => Some(Ordering::Less),

              (LowerBound::Excluded(l), UpperBound::Excluded(r))
            | (LowerBound::Excluded(l), UpperBound::Included(r))
            | (LowerBound::Included(l), UpperBound::Excluded(r)) =>
                match l.cmp(r) {
                    Ordering::Equal => Some(Ordering::Greater),
                    o => Some(o),
                },

            (LowerBound::Included(l), UpperBound::Included(r)) =>
                match l.cmp(r) {
                    Ordering::Equal => None,
                    o => Some(o),
                },
        }
    }
}

impl <T> PartialOrd<LowerBound<T>> for UpperBound<T> where T : Ord {
    fn partial_cmp(&self, other: &LowerBound<T>) -> Option<Ordering> {
        if let Some(x) = other.partial_cmp(self) {
            Some(x.reverse())
        }
        else {
            None
        }
    }
}

/// Conversion from std::ops::Bound to a lower bound
impl <T> From<std::ops::Bound<T>> for LowerBound<T> {
    fn from(bound: std::ops::Bound<T>) -> Self {
        match bound {
            std::ops::Bound::Excluded(x) => LowerBound::Excluded(x),
            std::ops::Bound::Included(x) => LowerBound::Included(x),
            std::ops::Bound::Unbounded => LowerBound::Unbounded,
        }
    }
}

/// Conversion from std::ops::Bound to an upper bound
impl <T> From<std::ops::Bound<T>> for UpperBound<T> {
    fn from(bound: std::ops::Bound<T>) -> Self {
        match bound {
            std::ops::Bound::Excluded(x) => UpperBound::Excluded(x),
            std::ops::Bound::Included(x) => UpperBound::Included(x),
            std::ops::Bound::Unbounded => UpperBound::Unbounded,
        }
    }
}

/**
 * Check if bounds are touching.
 */

impl <T> LowerBound<T> where T : PartialEq {
    pub fn is_touching(&self, other: &UpperBound<T>) -> bool {
        match (self, other) {
              (LowerBound::Excluded(x), UpperBound::Included(y))
            | (LowerBound::Included(x), UpperBound::Excluded(y)) => x.eq(y),

            _ => false,
        }
    }
}

impl <T> UpperBound<T> where T : PartialEq {
    pub fn is_touching(&self, other: &LowerBound<T>) -> bool {
        other.is_touching(self)
    }
}

/**
 * Construct a touching bound pair.
 */

impl <T> LowerBound<T> where T : Clone {
    pub fn touching(&self) -> Option<UpperBound<T>> {
        match self {
            LowerBound::Unbounded => None,
            LowerBound::Excluded(x) => Some(UpperBound::Included(x.clone())),
            LowerBound::Included(x) => Some(UpperBound::Excluded(x.clone())),
        }
    }
}

impl <T> UpperBound<T> where T : Clone {
    pub fn touching(&self) -> Option<LowerBound<T>> {
        match self {
            UpperBound::Unbounded => None,
            UpperBound::Excluded(x) => Some(LowerBound::Included(x.clone())),
            UpperBound::Included(x) => Some(LowerBound::Excluded(x.clone())),
        }
    }
}
