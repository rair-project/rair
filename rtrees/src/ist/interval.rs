/**
 * interval.rs: Auxilary Interval data structure for IST.
 *  Copyright (C) 2019  Oddcoder
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 **/
use std::cmp::{max, min};
#[derive(Clone, Copy)]
pub struct Interval2<T: Ord + Copy> {
    pub lo: T,
    pub hi: T,
    pub min_lo: T,
    pub max_hi: T,
}

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
