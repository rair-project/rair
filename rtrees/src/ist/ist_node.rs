/**
 * ist_node.rs: Augmented Interval Search Tree node implementation.
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
use super::interval::Interval;
use std::cmp::{max, min};
use std::mem;
pub struct ISTNode<K: Ord + Copy, V> {
    lo: K,
    hi: K,
    max_hi: K, //[lo, hi] It is closed interval
    min_lo: K,
    pub level: u64,
    right: SubTree<K, V>,
    left: SubTree<K, V>,
    data: V,
}

pub type SubTree<K, V> = Option<Box<ISTNode<K, V>>>;

macro_rules! node_has_point {
    ($node: expr, $point: expr) => {
        $point >= $node.lo && $point <= $node.hi
    };
}
macro_rules! interval_has_node {
    ($node: expr, $lo: expr, $hi: expr) => {
        $lo <= $node.lo && $hi >= $node.hi
    };
}

macro_rules! node_is_overlapping {
    ($node: expr, $lo: expr, $hi: expr) => {
        max($lo, $node.lo) <= min($hi, $node.hi)
    };
}

fn tear_leftmost<K: Ord + Copy, V>(root: &mut SubTree<K, V>) -> (SubTree<K, V>, SubTree<K, V>) {
    if root.as_ref().unwrap().left.is_none() {
        let place_me = root.as_mut().unwrap().right.take();
        let leftmost = root.take();
        return (leftmost, place_me);
    }
    let (leftmost, place_me) = tear_leftmost(&mut root.as_mut().unwrap().left);
    if let Some(new_left) = place_me {
        root.as_mut().unwrap().left.replace(new_left);
    }
    root.as_mut().unwrap().fix_aug_data();
    return (leftmost, None);
}

#[inline]
fn subtree_has_point<K: Ord + Copy, V>(subtree: &SubTree<K, V>, point: K) -> bool {
    subtree.is_some() && point >= subtree.as_ref().unwrap().min_lo && point <= subtree.as_ref().unwrap().max_hi
}
#[inline]
fn envelop_here<K: Ord + Copy, V>(subtree: &SubTree<K, V>, lo: K, hi: K) -> bool {
    subtree.is_some() && hi <= subtree.as_ref().unwrap().max_hi && lo >= subtree.as_ref().unwrap().min_lo
}

#[inline]
fn subtree_overlap<K: Ord + Copy, V>(subtree: &SubTree<K, V>, lo: K, hi: K) -> bool {
    subtree.is_some() && max(subtree.as_ref().unwrap().min_lo, lo) <= min(subtree.as_ref().unwrap().max_hi, hi)
}
impl<K: Ord + Copy, V> ISTNode<K, V> {
    pub fn new(lo: K, hi: K, data: V) -> SubTree<K, V> {
        return Some(Box::new(ISTNode {
            lo,
            hi,
            max_hi: hi,
            min_lo: lo,
            right: None,
            left: None,
            data,
            level: 1,
        }));
    }
    fn fix_aug_data(&mut self) {
        let mut max_hi = self.hi;
        let mut min_lo = self.lo;
        let mut level = 0;
        if let Some(right) = self.right.as_ref() {
            max_hi = max(max_hi, right.max_hi);
            min_lo = min(min_lo, right.min_lo);
            level = right.level;
        }
        if let Some(left) = self.left.as_ref() {
            max_hi = max(max_hi, left.max_hi);
            min_lo = min(min_lo, left.min_lo);
            level = max(level, left.level);
        }
        self.max_hi = max_hi;
        self.min_lo = min_lo;
        self.level = level + 1;
    }
    pub fn insert(&mut self, lo: K, hi: K, data: V) {
        if lo < self.lo || (lo == self.lo && hi <= self.hi) {
            match &mut self.left {
                Some(node) => node.insert(lo, hi, data),
                None => self.left = ISTNode::new(lo, hi, data),
            }
        } else {
            match &mut self.right {
                Some(node) => node.insert(lo, hi, data),
                None => self.right = ISTNode::new(lo, hi, data),
            }
        }
        self.fix_aug_data();
    }
    fn generic_search<F: Fn(&SubTree<K, V>, K, K) -> bool, T: Fn(&Interval<K>, K, K) -> bool>(&self, lo: K, hi: K, recurse: &F, accept: &T) -> Vec<&V> {
        let mut result = if recurse(&self.left, lo, hi) {
            self.left.as_ref().unwrap().generic_search(lo, hi, recurse, accept)
        } else {
            Vec::new()
        };
        let interval = Interval {
            lo: self.lo,
            hi: self.hi,
            min_lo: self.min_lo,
            max_hi: self.max_hi,
        };

        if accept(&interval, lo, hi) {
            result.push(&self.data);
        }
        if recurse(&self.right, lo, hi) {
            result.extend(self.right.as_ref().unwrap().generic_search(lo, hi, recurse, accept));
        }
        return result;
    }
    fn generic_search_mut<F: Fn(&SubTree<K, V>, K, K) -> bool, T: Fn(&Interval<K>, K, K) -> bool>(&mut self, lo: K, hi: K, recurse: &F, accept: &T) -> Vec<&mut V> {
        let mut result = if recurse(&self.left, lo, hi) {
            self.left.as_mut().unwrap().generic_search_mut(lo, hi, recurse, accept)
        } else {
            Vec::new()
        };
        let interval = Interval {
            lo: self.lo,
            hi: self.hi,
            min_lo: self.min_lo,
            max_hi: self.max_hi,
        };
        if accept(&interval, lo, hi) {
            result.push(&mut self.data);
        }
        if recurse(&self.right, lo, hi) {
            result.extend(self.right.as_mut().unwrap().generic_search_mut(lo, hi, recurse, accept));
        }
        return result;
    }
    pub fn at(&self, point: K) -> Vec<&V> {
        let accept = |interval: &Interval<K>, point: K, _: K| node_has_point!(interval, point);
        let recurse = |subtree: &SubTree<K, V>, point: K, _: K| subtree_has_point(subtree, point);
        return self.generic_search(point, point, &recurse, &accept);
    }
    pub fn at_mut(&mut self, point: K) -> Vec<&mut V> {
        let accept = |interval: &Interval<K>, point: K, _: K| node_has_point!(interval, point);
        let recurse = |subtree: &SubTree<K, V>, point: K, _: K| subtree_has_point(subtree, point);
        return self.generic_search_mut(point, point, &recurse, &accept);
    }
    pub fn envelop(&self, lo: K, hi: K) -> Vec<&V> {
        let accept = |interval: &Interval<K>, lo: K, hi: K| node_has_point!(interval, lo) && node_has_point!(interval, hi);
        return self.generic_search(lo, hi, &envelop_here, &accept);
    }
    pub fn envelop_mut(&mut self, lo: K, hi: K) -> Vec<&mut V> {
        let accept = |interval: &Interval<K>, lo: K, hi: K| node_has_point!(interval, lo) && node_has_point!(interval, hi);
        return self.generic_search_mut(lo, hi, &envelop_here, &accept);
    }
    pub fn inverse_envelop(&self, lo: K, hi: K) -> Vec<&V> {
        // I couldn't find the right equation for inverse envelope subtrees
        // but if they are not intersection anyway they can't have any kind of
        // envelopment relation ship
        let accept = |interval: &Interval<K>, lo: K, hi: K| interval_has_node!(interval, lo, hi);
        return self.generic_search(lo, hi, &subtree_overlap, &accept);
    }
    pub fn inverse_envelop_mut(&mut self, lo: K, hi: K) -> Vec<&mut V> {
        // I couldn't find the right equation for inverse envelope subtrees
        // but if they are not intersection anyway they can't have any kind of
        // envelopment relation ship
        let accept = |interval: &Interval<K>, lo: K, hi: K| interval_has_node!(interval, lo, hi);
        return self.generic_search_mut(lo, hi, &subtree_overlap, &accept);
    }

    pub fn overlap(&self, lo: K, hi: K) -> Vec<&V> {
        let accept = |interval: &Interval<K>, lo: K, hi: K| node_is_overlapping!(interval, lo, hi);
        return self.generic_search(lo, hi, &subtree_overlap, &accept);
    }
    pub fn overlap_mut(&mut self, lo: K, hi: K) -> Vec<&mut V> {
        let accept = |interval: &Interval<K>, lo: K, hi: K| node_is_overlapping!(interval, lo, hi);
        return self.generic_search_mut(lo, hi, &subtree_overlap, &accept);
    }
    fn delete_node_with_both_children(mut self) -> (SubTree<K, V>, ISTNode<K, V>) {
        let left_subtree = mem::replace(&mut self.left, None);
        let mut right_subtree = mem::replace(&mut self.right, None);
        let (mut leftmost_of_right_subtree, place_me) = tear_leftmost(&mut right_subtree);
        if place_me.is_some() {
            right_subtree = place_me;
        }
        leftmost_of_right_subtree.as_mut().unwrap().left = left_subtree;
        leftmost_of_right_subtree.as_mut().unwrap().right = right_subtree;
        return (leftmost_of_right_subtree, self);
    }
    /// returns the SubTree to replace the current one and the deleted
    /// node for after being totally unlinked for purposes of extracting
    /// data.
    fn delete_node(mut self) -> (SubTree<K, V>, ISTNode<K, V>) {
        if self.right.is_none() && self.left.is_none() {
            return (None, self);
        } else if self.right.is_none() {
            return (self.left.take(), self);
        } else if self.left.is_none() {
            return (self.right.take(), self);
        } else {
            return self.delete_node_with_both_children();
        }
    }

    fn generic_delete<F: Fn(&SubTree<K, V>, K, K) -> bool, T: Fn(&ISTNode<K, V>, K, K) -> bool>(mut self, lo: K, hi: K, recurse: &F, delete: &T) -> (SubTree<K, V>, Vec<V>) {
        let mut deleted = if recurse(&self.left, lo, hi) {
            let left_subtree = mem::replace(&mut self.left, None);
            let x = left_subtree.unwrap().generic_delete(lo, hi, recurse, delete);
            self.left = x.0;
            x.1
        } else {
            Vec::new()
        };
        let tmp = if recurse(&self.right, lo, hi) {
            let right_subtree = mem::replace(&mut self.right, None);
            let x = right_subtree.unwrap().generic_delete(lo, hi, recurse, delete);
            self.right = x.0;
            x.1
        } else {
            Vec::new()
        };
        self.fix_aug_data();
        let node = if delete(&self, lo, hi) {
            let x = self.delete_node();
            deleted.push(x.1.data);
            x.0
        } else {
            Some(Box::new(self))
        };
        deleted.extend(tmp);
        return (node, deleted);
    }
    pub fn delete_at(self, point: K) -> (SubTree<K, V>, Vec<V>) {
        let delete = |node: &ISTNode<K, V>, point: K, _ignored: K| node_has_point!(node, point);
        let recurse = |subtree: &SubTree<K, V>, point: K, _ignored: K| subtree_has_point(subtree, point);
        return self.generic_delete(point, point, &recurse, &delete);
    }
    pub fn delete_envelop(self, lo: K, hi: K) -> (SubTree<K, V>, Vec<V>) {
        let delete = |node: &ISTNode<K, V>, lo: K, hi: K| node_has_point!(node, lo) && node_has_point!(node, hi);
        return self.generic_delete(lo, hi, &envelop_here, &delete);
    }
    pub fn delete_overlap(self, lo: K, hi: K) -> (SubTree<K, V>, Vec<V>) {
        let delete = |node: &ISTNode<K, V>, lo: K, hi: K| node_is_overlapping!(node, lo, hi);
        return self.generic_delete(lo, hi, &subtree_overlap, &delete);
    }
}
