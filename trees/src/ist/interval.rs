//! Auxilary closed Interval data structure for IST.

use core::cmp::{max, min};

#[derive(Clone, Copy, Default, Ord, Eq, PartialOrd, PartialEq)]
pub(super) struct Interval<T: Ord + Copy> {
    pub(super) lo: T,
    pub(super) hi: T,
}

impl<T: Ord + Copy> Interval<T> {
    pub(super) fn new(lo: T, hi: T) -> Interval<T> {
        Interval { lo, hi }
    }
    pub(super) fn absorb(&mut self, int: Interval<T>) {
        self.lo = min(self.lo, int.lo);
        self.hi = max(self.hi, int.hi);
    }
    pub(super) fn has_point(&self, point: T) -> bool {
        point >= self.lo && point <= self.hi
    }
    pub(super) fn envelop(&self, small: &Interval<T>) -> bool {
        self.has_point(small.lo) && self.has_point(small.hi)
    }
    pub(super) fn overlap(&self, int: &Interval<T>) -> bool {
        max(self.lo, int.lo) <= min(self.hi, int.hi)
    }
}
