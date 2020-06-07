/*
 * iter_ref.rs: similar to iter but it iterates over reference instead of consuming the tree.
 * Copyright (C) 2019  Oddcoder
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
 */

use super::interval::*;
use super::rb_helpers::*;
use super::tree::*;
use rbtree::TreeRefIterator;
use std::slice::Iter;
pub struct ISTRefIterator<'a, K: Ord + Copy, V> {
    tree_iter: TreeRefIterator<'a, Interval<K>, AugData<K>, Vec<V>>,
    lo: Option<K>,
    hi: Option<K>,
    current_iter: Iter<'a, V>,
}

impl<'a, K: Ord + Copy, V> ISTRefIterator<'a, K, V> {
    pub(crate) fn new(root: &'a IST<K, V>) -> ISTRefIterator<K, V> {
        ISTRefIterator {
            tree_iter: (&root.root).into_iter(),
            lo: None,
            hi: None,
            current_iter: [].iter(),
        }
    }
}
impl<'a, K: Ord + Copy, V> Iterator for ISTRefIterator<'a, K, V> {
    type Item = (K, K, &'a V);

    fn next(&mut self) -> Option<(K, K, &'a V)> {
        if let Some(data) = self.current_iter.next() {
            return Some((self.lo.unwrap(), self.hi.unwrap(), data));
        }
        if let Some((k, _, v)) = self.tree_iter.next() {
            self.current_iter = v.iter();
            self.lo = Some(k.lo);
            self.hi = Some(k.hi);
        } else {
            return None;
        }
        Some((self.lo.unwrap(), self.hi.unwrap(), self.current_iter.next().unwrap()))
    }
}
