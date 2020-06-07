/**
 * rb_helpers.rs: Extend RBTree functionality so it suits IST.
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
use super::interval::*;
use crate::rbtree::RBTree;

#[derive(Copy, Clone)]
pub(super) struct AugData<K: Ord + Copy> {
    pub(super) interval: Interval<K>,
    pub(super) size: u64,
}
impl<K: Ord + Copy> AugData<K> {
    pub fn new(interval: Interval<K>, size: u64) -> AugData<K> {
        AugData { interval, size }
    }
}

// Decision function:
// In case of the accept function First argument is the node key, second argument is search key.
// In case of the recurse function First argument is the node augmented data and second argument is the search key.
type Accept<K> = dyn Fn(&Interval<K>, &Interval<K>) -> bool;
type Recurse<K> = dyn Fn(&AugData<K>, &Interval<K>) -> bool;

pub(super) trait ISTHelpers<K: Ord + Copy, V> {
    fn generic_search(&self, int: Interval<K>, recurse: &Recurse<K>, accept: &Accept<K>) -> Vec<&V>;
    fn generic_search_mut(&mut self, int: Interval<K>, recurse: &Recurse<K>, accept: &Accept<K>) -> Vec<&mut V>;
    fn generic_key_search(&self, int: Interval<K>, recurse: &Recurse<K>, accept: &Accept<K>) -> Vec<Interval<K>>;
    fn generic_delete(&mut self, int: Interval<K>, recurse: &Recurse<K>, accept: &Accept<K>) -> Vec<V>;
}
impl<K: Ord + Copy, V> ISTHelpers<K, V> for RBTree<Interval<K>, AugData<K>, Vec<V>> {
    fn generic_search(&self, int: Interval<K>, recurse: &Recurse<K>, accept: &Accept<K>) -> Vec<&V> {
        let mut result = if self.left_ref().is_node() && recurse(&self.left_ref().aug_data(), &int) {
            self.left_ref().generic_search(int, recurse, accept)
        } else {
            Vec::new()
        };
        if accept(&self.key(), &int) {
            result.extend(self.data_ref().iter());
        }
        if self.right_ref().is_node() && recurse(&self.right_ref().aug_data(), &int) {
            result.extend(self.right_ref().generic_search(int, recurse, accept));
        }
        result
    }
    fn generic_search_mut(&mut self, int: Interval<K>, recurse: &Recurse<K>, accept: &Accept<K>) -> Vec<&mut V> {
        let key = self.key();
        let (left, right, data) = self.mut_me();
        let mut result = if left.is_node() && recurse(&left.aug_data(), &int) {
            left.generic_search_mut(int, recurse, accept)
        } else {
            Vec::new()
        };
        if accept(&key, &int) {
            result.extend(data.iter_mut());
        }
        if right.is_node() && recurse(&right.aug_data(), &int) {
            result.extend(right.generic_search_mut(int, recurse, accept));
        }
        result
    }
    fn generic_key_search(&self, int: Interval<K>, recurse: &Recurse<K>, accept: &Accept<K>) -> Vec<Interval<K>> {
        let mut keys = if self.left_ref().is_node() && recurse(&self.left_ref().aug_data(), &int) {
            self.left_ref().generic_key_search(int, recurse, accept)
        } else {
            Vec::new()
        };
        if accept(&self.key(), &int) {
            keys.push(self.key());
        }
        if self.right_ref().is_node() && recurse(&self.right_ref().aug_data(), &int) {
            keys.extend(self.right_ref().generic_key_search(int, recurse, accept));
        }
        keys
    }
    fn generic_delete(&mut self, int: Interval<K>, recurse: &Recurse<K>, accept: &Accept<K>) -> Vec<V> {
        let delete_keys = self.generic_key_search(int, recurse, accept);
        let mut result = Vec::with_capacity(delete_keys.len());
        for key in delete_keys {
            // we can safely unwrap because we already queried the keys!
            result.extend(self.delete(key).unwrap());
        }
        result
    }
}
