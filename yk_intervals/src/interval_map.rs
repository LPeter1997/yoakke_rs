
use crate::interval::{Interval};

pub struct IntervalMap<K, V> {
    intervals: Vec<(Interval<K>, V)>,
}

impl <K, V> IntervalMap<K, V> {
    pub fn new() -> Self {
        IntervalMap{ intervals: Vec::new() }
    }
}

// We need this because the standard library doesn't deal with different key types
fn bsearch_by_key<'a, T, K1, K2, F>(slice: &'a [T], searched: &K1, mut key_selector: F) -> Result<usize, usize>
    where F : FnMut(&'a T) -> K2, K1 : PartialOrd<K2> {

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
        Err(base + (cmp == std::cmp::Ordering::Less) as usize)
    }
}

impl <K, V> IntervalMap<K, V> where K : Ord + Clone {
    pub fn intersecting_index_range(&self, interval: &Interval<K>) -> std::ops::Range<usize> {
        let from = match bsearch_by_key(&self.intervals[..], &interval.lower, |x| x.0.upper.clone()) {
            Ok(idx) | Err(idx) => idx
        };
        let to = match bsearch_by_key(&self.intervals[from..], &interval.upper, |x| x.0.lower.clone()) {
            Ok(idx) | Err(idx) => idx
        } + from;

        from..to
    }

    pub fn touching_index_range(&self, interval: &Interval<K>) -> std::ops::Range<usize> {
        let std::ops::Range{ mut start, mut end, } = self.intersecting_index_range(interval);

        if start != 0 && interval.lower.is_touching(&self.intervals[start - 1].0.upper) {
            start -= 1;
        }
        if end != self.intervals.len() && interval.upper.is_touching(&self.intervals[end].0.lower) {
            end += 1;
        }

        start..end
    }
}
