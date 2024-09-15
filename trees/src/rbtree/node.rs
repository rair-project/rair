//! Non Empty node Implementation.

use super::color::Color;
use super::rbtree_wrapper::{Augment, RBTree};
use core::cmp::max;
pub(super) struct Node<K: Ord + Copy, A: Copy, V> {
    pub(super) key: K,
    pub(super) aug_data: A,
    pub(super) data: V,
    pub(super) level: u64,
    size: u64,
    pub(super) color: Color,
    pub(super) left: RBTree<K, A, V>,
    pub(super) right: RBTree<K, A, V>,
}

impl<K: Ord + Copy, A: Copy, V> Node<K, A, V>
where
    RBTree<K, A, V>: Augment<A>,
{
    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn get_level(&self) -> u64 {
        self.level
    }
    pub fn new(key: K, aug_data: A, data: V) -> Self {
        Node {
            key,
            aug_data,
            data,
            level: 1,
            size: 1,
            color: Color::Red,
            left: RBTree::new(),
            right: RBTree::new(),
        }
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
        self.color = Color::Red;
        x.as_mut().unwrap().left = self.into();
        x.as_mut().unwrap().left.sync_aug();
        x.sync_aug();
        x.unwrap()
    }
    pub(super) fn rotate_right(mut self) -> Self {
        let mut x = self.left.take();
        self.left = x.as_mut().unwrap().right.take();
        x.as_mut().unwrap().color = self.color;
        self.color = Color::Red;
        x.as_mut().unwrap().right = self.into();
        x.as_mut().unwrap().right.sync_aug();
        x.sync_aug();
        x.unwrap()
    }
    #[inline]
    pub(super) fn flip_colors(&mut self) {
        self.color.flip();
        self.left.as_mut().unwrap().color.flip();
        self.right.as_mut().unwrap().color.flip();
    }
    #[inline]
    pub(super) fn is_red(&self) -> bool {
        self.color == Color::Red
    }
    pub(super) fn move_red_left(mut self) -> Self {
        self.flip_colors();
        if self.right.as_ref().unwrap().left.is_red() {
            self.right = self.right.unwrap().rotate_right().into();
            self = self.rotate_left();
            self.flip_colors();
        }
        self
    }
    pub(super) fn move_red_right(mut self) -> Self {
        assert!(self.is_red());
        self.flip_colors();
        if self.left.as_ref().unwrap().left.is_red() {
            self = self.rotate_right();
            self.flip_colors();
        }
        self
    }
}
