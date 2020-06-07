/*
 * iter.rs: Iterator implementation for rbtree.
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
// Credits where credits goes!
// https://codereview.stackexchange.com/questions/110161/binary-trees-in-rust-iterators
use super::rbtree_wrapper::{Augment, RBTree};

/// Iterator for [RBtree]
pub struct TreeIterator<K: Ord + Copy, A: Copy, V> {
    right: Vec<RBTree<K, A, V>>,
    current: Option<RBTree<K, A, V>>,
}

impl<K: Ord + Copy, A: Copy, V> TreeIterator<K, A, V>
where
    RBTree<K, A, V>: Augment<A>,
{
    pub(crate) fn new(root: RBTree<K, A, V>) -> TreeIterator<K, A, V> {
        let mut iter = TreeIterator { right: vec![], current: None };
        iter.add_subtree(root);
        iter
    }
    fn add_subtree(&mut self, root: RBTree<K, A, V>) {
        let mut node: RBTree<K, A, V> = root;
        while node.is_node() {
            if node.right_ref().is_node() {
                self.right.push(node.right());
            }
            if node.left_ref().is_node() {
                let tmp = node.left();
                self.right.push(node);
                node = tmp;
            } else {
                break;
            }
        }
        self.current = if node.is_node() { Some(node) } else { None };
    }
}
impl<K: Ord + Copy, A: Copy, V> Iterator for TreeIterator<K, A, V>
where
    RBTree<K, A, V>: Augment<A>,
{
    type Item = (K, A, V);

    fn next(&mut self) -> Option<(K, A, V)> {
        let result;
        if let Some(node) = self.current.take() {
            result = Some((node.key(), node.aug_data(), node.data()));
        } else {
            return None;
        }
        if let Some(node) = self.right.pop() {
            self.add_subtree(node);
        }
        result
    }
}
