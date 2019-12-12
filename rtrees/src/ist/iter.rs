/*
 * iter.rs: Iterator implementation for ist.
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
use rbtree::TreeIterator;
use std::vec::IntoIter;

pub struct ISTIterator<K: Ord + Copy, V> {
    tree_iter: TreeIterator<Interval<K>, AugData<K>, Vec<V>>,
    current_iter: IntoIter<V>,
}

impl<K: Ord + Copy, V> ISTIterator<K, V> {
    pub(crate) fn new(root: IST<K, V>) -> ISTIterator<K, V> {
        ISTIterator {
            tree_iter: root.root.into_iter(),
            current_iter: Vec::new().into_iter(),
        }
    }
}
impl<K: Ord + Copy, V> Iterator for ISTIterator<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<V> {
        if let Some(data) = self.current_iter.next() {
            return Some(data);
        }
        if let Some((_, _, v)) = self.tree_iter.next() {
            self.current_iter = v.into_iter();
        } else {
            return None;
        }
        return self.current_iter.next();
    }
}
