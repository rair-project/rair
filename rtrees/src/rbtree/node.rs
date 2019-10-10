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
pub(super) struct Node<K: Ord + Copy, V> {
    pub(super) key: K,
    pub(super) data: V,
    pub(super) level: u64,
    size: u64,
    pub(super) color: COLOR,
    pub(super) left: RBTree<K, V>,
    pub(super) right: RBTree<K, V>,
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
    pub(super) fn sync_builtin_aug(&mut self) {
        self.size = self.left.size() + self.right.size() + 1;
        self.level = max(self.left.get_level(), self.right.get_level()) + 1;
    }
    // We don't check if this node is valid for the operation
    // or not in the case of rotate_left and rotate_right, and
    // flip_colors. normally user will only use them when they are needed.
    pub(super) fn rotate_left(mut self) -> Self {
        let mut x = self.right.take();
        self.right = x.as_mut().unwrap().left.take();
        x.as_mut().unwrap().color = self.color;
        self.color = COLOR::RED;
        self.sync_builtin_aug();
        x.as_mut().unwrap().left = self.into();
        let mut new_node = x.unwrap();
        new_node.sync_builtin_aug();
        return *new_node;
    }
    pub(super) fn rotate_right(mut self) -> Self {
        let mut x = self.left.take();
        self.left = x.as_mut().unwrap().right.take();
        x.as_mut().unwrap().color = self.color;
        self.color = COLOR::RED;
        self.sync_builtin_aug();
        x.as_mut().unwrap().right = self.into();
        let mut new_node = x.unwrap();
        new_node.sync_builtin_aug();
        return *new_node;
    }
    #[inline]
    pub(super) fn flip_colors(&mut self) {
        self.color.flip();
        self.left.as_mut().unwrap().color.flip();
        self.right.as_mut().unwrap().color.flip();
    }
    #[inline]
    pub(super) fn is_red(&self) -> bool {
        self.color == COLOR::RED
    }
    pub(super) fn move_red_left(mut self) -> Self {
        self.flip_colors();
        if self.right.as_ref().unwrap().left.is_red() {
            self.right = self.right.unwrap().rotate_right().into();
            self = self.rotate_left();
            self.flip_colors();
        }
        return self;
    }
    pub(super) fn move_red_right(mut self) -> Self {
        assert!(self.is_red());
        self.flip_colors();
        if self.left.as_ref().unwrap().left.is_red() {
            self = self.rotate_right();
            self.flip_colors();
        }
        return self;
    }
}
