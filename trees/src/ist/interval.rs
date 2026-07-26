//! Auxilary closed Interval data structure for IST.

use core::cmp::{max, min, Ordering};

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub(super) struct Interval<T: Ord + Copy> {
    pub(super) hi: T,
    pub(super) lo: T,
}

impl<T: Ord + Copy> Ord for Interval<T> {
    /// Compare by `lo` first, then by `hi`. This preserves the ordering
    /// semantics of the previously derived implementation (which compared
    /// fields in `(lo, hi)` declaration order).
    fn cmp(&self, other: &Self) -> Ordering {
        self.lo.cmp(&other.lo).then_with(|| self.hi.cmp(&other.hi))
    }
}

impl<T: Ord + Copy> PartialOrd for Interval<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord + Copy> Interval<T> {
    pub(super) fn absorb(&mut self, int: Interval<T>) {
        self.lo = min(self.lo, int.lo);
        self.hi = max(self.hi, int.hi);
    }
    pub(super) fn envelop(&self, small: &Interval<T>) -> bool {
        self.has_point(small.lo) && self.has_point(small.hi)
    }
    pub(super) fn has_point(&self, point: T) -> bool {
        point >= self.lo && point <= self.hi
    }
    pub(super) fn new(lo: T, hi: T) -> Interval<T> {
        Interval { hi, lo }
    }
    pub(super) fn overlap(&self, int: &Interval<T>) -> bool {
        max(self.lo, int.lo) <= min(self.hi, int.hi)
    }
}
