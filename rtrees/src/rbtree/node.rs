/*
 * rbtree.rs: Non Empty node Implementation.
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

use super::color::COLOR;
use super::rbtree_wrapper::RBTree;
use std::cmp::max;
use std::mem;
pub struct Node<K: Ord + Copy, V> {
    pub(super) key: K,
    pub(super) data: V,
    pub(super) level: u64,
    size: u64,
    pub(super) color: COLOR,
    pub(super) left: RBTree<K, V>,
    pub(super) right: RBTree<K, V>,
}

fn node_min<K: Ord + Copy, V>(root: &mut RBTree<K, V>) -> &mut RBTree<K, V> {
    if root.left_ref().is_node() {
        return node_min(root.left_mut());
    } else {
        return root;
    }
}

impl<K: Ord + Copy, V> Node<K, V> {
    pub fn size(&self) -> u64 {
        return self.size;
    }
    pub fn get_level(&self) -> u64 {
        return self.level;
    }
    pub fn new(key: K, data: V) -> Self {
        return Node {
            key,
            data,
            level: 1,
            size: 1,
            color: COLOR::RED,
            left: RBTree::new(),
            right: RBTree::new(),
        };
    }
    //sync augmented data
    fn sync_aug(&mut self) {
        self.size = self.left.size() + self.right.size() + 1;
        self.level = max(self.left.get_level(), self.right.get_level()) + 1;
    }
    // We don't check if this node is valid for the operation
    // or not in the case of rotate_left and rotate_right, and
    // flip_colors. normally user will only use them when they are needed.
    fn rotate_left(mut self) -> Self {
        let mut x = self.right.take();
        self.right = x.as_mut().unwrap().left.take();
        x.as_mut().unwrap().color = self.color;
        self.color = COLOR::RED;
        self.sync_aug();
        x.as_mut().unwrap().left = self.into();
        let mut new_node = x.unwrap();
        new_node.sync_aug();
        return *new_node;
    }
    fn rotate_right(mut self) -> Self {
        let mut x = self.left.take();
        self.left = x.as_mut().unwrap().right.take();
        x.as_mut().unwrap().color = self.color;
        self.color = COLOR::RED;
        self.sync_aug();
        x.as_mut().unwrap().right = self.into();
        let mut new_node = x.unwrap();
        new_node.sync_aug();
        return *new_node;
    }
    #[inline]
    fn flip_colors(&mut self) {
        self.color.flip();
        self.left.as_mut().unwrap().color.flip();
        self.right.as_mut().unwrap().color.flip();
    }
    #[inline]
    pub(super) fn is_red(&self) -> bool {
        self.color == COLOR::RED
    }
    fn balance(mut self) -> Self {
        self.sync_aug();
        if self.right.is_red() && !self.left.is_red() {
            self = self.rotate_left();
        }
        if self.left.is_red() && self.left.as_ref().unwrap().left.is_red() {
            self = self.rotate_right();
        }
        if self.left.is_red() && self.right.is_red() {
            self.flip_colors();
        }
        return self;
    }
    pub fn insert(mut self, key: K, data: V) -> Self {
        if key < self.key {
            self.left = self.left.insert_random_node(key, data);
        } else if key > self.key {
            self.right = self.right.insert_random_node(key, data);
        } else {
            self.data = data;
        }
        self = self.balance();
        return self;
    }

    fn move_red_left(mut self) -> Self {
        self.flip_colors();
        if self.right.as_ref().unwrap().left.is_red() {
            self.right = self.right.unwrap().rotate_right().into();
            self = self.rotate_left();
            self.flip_colors();
        }
        return self;
    }
    fn move_red_right(mut self) -> Self {
        assert!(self.is_red());
        self.flip_colors();
        if self.left.as_ref().unwrap().left.is_red() {
            self = self.rotate_right();
            self.flip_colors();
        }
        return self;
    }
    pub(super) fn delete_min_not_root(mut self) -> (RBTree<K, V>, Option<V>) {
        if !self.left.is_red() && !self.left.as_ref().unwrap().left.is_red() {
            self = self.move_red_left();
        }
        let result = self.left.delete_min_not_root();
        self.left = result.0;
        self = self.balance();

        return (self.into(), result.1);
    }
    pub fn delete(mut self, key: K) -> (RBTree<K, V>, Option<V>) {
        let mut result;
        if key < self.key {
            if !self.left.is_red() && self.left.is_node() && !self.left.as_ref().unwrap().left.is_red() {
                self = self.move_red_left();
            }
            result = self.left.delete_random_node(key);
            self.left = result.0;
            result.0 = self.into();
        } else {
            if self.left.is_red() {
                self = self.rotate_right();
            }
            if key == self.key && !self.right.is_node() {
                return (RBTree::new(), Some(self.data));
            }
            if !self.right.is_red() && self.right.is_node() && !self.right.as_ref().unwrap().left.is_red() {
                self = self.move_red_right();
            }
            if self.key == key {
                let x = node_min(&mut self.right).as_mut().unwrap();
                self.key = x.key;
                mem::swap(&mut self.data, &mut x.data);
                result = self.right.delete_min_not_root();
                self.right = result.0;
                result.0 = self.into();
            } else {
                result = self.right.delete_random_node(key);
                self.right = result.0;
                result.0 = self.into();
            }
        }
        result.0 = result.0.unwrap().balance().into();
        return result;
    }
}
