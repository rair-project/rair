/*
 * rbtree.rs: Tree wrapper.
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
use super::iter::TreeIterator;
use super::iter_ref::TreeRefIterator;
use super::node::*;
use std::cmp::Ordering;
use std::mem;

pub(super) type RBTreeOp<K, A, V> = Option<Box<Node<K, A, V>>>;

/// Tuple of 3 elements used with [RBTree::mut_me]. The first element is mutable reference to the left subtree,
/// the second element is a mutable reference to the right subtree and the third element is a mutable reference
/// to the data stored in the current node.
pub type LeftRightDataTuple<'a, K, A, V> = (&'a mut RBTree<K, A, V>, &'a mut RBTree<K, A, V>, &'a mut V);

///  Used to recalculate augmented data stored in each node.
/// This trait is mainly meant to be only implemented for [RBTree]
/// before using the tree.
pub trait Augment<T: Copy> {
    fn sync_custom_aug(&mut self) {}
}

/// A left-leaning red–black (LLRB) Tree, optimized for safety, simplicity of implementation,
/// and augmentation. This tree is mimicking the behavior of a 2-3 tree. Full desciption of the
/// tree design and complexity analysis is available in the paper titled
/// [*Left-leaning Red-Black Trees*](https://www.cs.princeton.edu/~rs/talks/LLRB/LLRB.pdf) by
/// Robert Sedgewick. If you just need a normal non-augmentable tree-baesed map check
/// [std::collections::BTreeMap] instead.
#[derive(Default)]
pub struct RBTree<K: Ord + Copy, A: Copy, V>(RBTreeOp<K, A, V>);

impl<K: Ord + Copy, A: Copy, V> From<Node<K, A, V>> for RBTree<K, A, V> {
    fn from(node: Node<K, A, V>) -> Self {
        RBTree(Some(Box::new(node)))
    }
}

impl<K: Ord + Copy, A: Copy, V> RBTree<K, A, V>
where
    RBTree<K, A, V>: Augment<A>,
{
    // Implementing wrapper for Option functionality
    #[inline]
    pub(super) fn take(&mut self) -> RBTree<K, A, V> {
        RBTree(self.0.take())
    }
    /// Return True if the current node is not null node.
    pub fn is_node(&self) -> bool {
        self.0.is_some()
    }

    #[inline]
    pub(super) fn as_mut(&mut self) -> Option<&mut Node<K, A, V>> {
        self.0.as_deref_mut()
    }

    #[inline]
    pub(super) fn as_ref(&self) -> Option<&Node<K, A, V>> {
        self.0.as_deref()
    }

    #[inline]
    pub(super) fn unwrap(self) -> Box<Node<K, A, V>> {
        self.0.unwrap()
    }
    #[inline]
    pub(super) fn is_red(&self) -> bool {
        self.is_node() && self.as_ref().unwrap().is_red()
    }

    /// Returns copy of key of the current Tree node
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn key(&self) -> K {
        self.as_ref().unwrap().key
    }

    /// Changes the *aug_data* stored in the current Tree node.
    /// # Panics
    /// panics if current subtree is not a *node*.
    pub fn set_aug_data(&mut self, aug_data: A) {
        self.as_mut().unwrap().aug_data = aug_data;
    }

    /// Returns *aug_data* stored in the current Tree node.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn aug_data(&self) -> A {
        self.as_ref().unwrap().aug_data
    }

    /// Changes the *data* stored in the current Tree node.
    /// # Panics
    /// panics if current subtree is not a *node*.
    pub fn set_data(&mut self, data: V) {
        self.as_mut().unwrap().data = data;
    }

    /// Returns data stored in the current Tree node.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn data(self) -> V {
        self.unwrap().data
    }
    /// Returns a tuple of tree elements: a mutable reference to left node,
    /// mutable right node and mutable referent to the value stored
    /// inside the current node. The reason such functionality might be
    /// desired, is when user wants to keep mutual reference of at
    /// least any 2 of either left node, right node or data.
    /// [left_mut()](struct.RBTree.html#method.left_mut),
    /// [right_mut()](struct.RBTree.html#method.right_mut)
    /// and [data_mut()](struct.RBTree.html#method.data_mut) will not
    /// work because rust does not support partial
    /// borrowing [yet](https://github.com/rust-lang/rfcs/issues/1215).
    pub fn mut_me(&mut self) -> LeftRightDataTuple<K, A, V> {
        let node = self.as_mut().unwrap();
        return (&mut node.left, &mut node.right, &mut node.data);
    }
    /// Returns non-mutable reference to data stored in the current Tree node
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn data_ref(&self) -> &V {
        &self.as_ref().unwrap().data
    }

    /// Returns mutable reference to data stored in the current Tree node
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn data_mut(&mut self) -> &mut V {
        &mut self.as_mut().unwrap().data
    }

    /// Set the left subtree of the current Node.
    /// # Panics
    /// panics if current subtree is not a *node*.
    pub fn set_left(&mut self, subtree: RBTree<K, A, V>) {
        self.as_mut().unwrap().left = subtree;
    }

    /// Returns the left subtree after ripping it from the current node.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn left(&mut self) -> RBTree<K, A, V> {
        self.as_mut().unwrap().left.take()
    }

    /// Returns a non-mutable reference to left subtree.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn left_ref(&self) -> &RBTree<K, A, V> {
        &self.as_ref().unwrap().left
    }

    /// Returns a mutable reference to left subtree.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn left_mut(&mut self) -> &mut RBTree<K, A, V> {
        &mut self.as_mut().unwrap().left
    }

    /// Set the right subtree of the current Node.
    /// # Panics
    /// panics if current subtree is not a *node*.
    pub fn set_right(&mut self, subtree: RBTree<K, A, V>) {
        self.as_mut().unwrap().right = subtree;
    }

    /// Returns the right subtree after ripping it from the current node.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn right(&mut self) -> RBTree<K, A, V> {
        self.as_mut().unwrap().right.take()
    }

    /// Returns a non-mutable reference to right subtree.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn right_ref(&self) -> &RBTree<K, A, V> {
        &self.as_ref().unwrap().right
    }

    /// Returns a mutable reference to right subtree.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn right_mut(&mut self) -> &mut RBTree<K, A, V> {
        &mut self.as_mut().unwrap().right
    }

    /// Returns new Red Black Tree
    /// # Example
    /// ```
    /// use rtrees::rbtree::*;
    /// #[derive(Copy, Clone)]
    /// struct PlaceHolder();
    /// impl Augment<PlaceHolder> for RBTree<u64, PlaceHolder, &'static str> {}
    /// type Tree = RBTree<u64, PlaceHolder,  &'static str>;
    /// let my_tree = Tree::new();
    /// ```
    pub fn new() -> RBTree<K, A, V> {
        RBTree(None)
    }

    /// Returns the number of elements in the tree
    /// # Example
    /// ```
    /// use rtrees::rbtree::*;
    /// #[derive(Copy, Clone)]
    /// struct PlaceHolder();
    /// impl Augment<PlaceHolder> for RBTree<u64, PlaceHolder, &'static str> {}
    /// type Tree = RBTree<u64, PlaceHolder,  &'static str>;
    /// let mut rbtree = Tree::new();
    /// assert_eq!(rbtree.size(), 0);
    /// rbtree.insert(0, PlaceHolder(), "Zero");
    /// assert_eq!(rbtree.size(), 1);
    /// rbtree.insert(1, PlaceHolder(), "One");
    /// assert_eq!(rbtree.size(), 2);
    /// rbtree.insert(2, PlaceHolder(), "Two");
    /// assert_eq!(rbtree.size(), 3);
    /// ```
    pub fn size(&self) -> u64 {
        return if let Some(node) = &self.0 { node.size() } else { 0 };
    }

    /// 0 will be returned in case of empty tree. If tree has nodes, then *get_level*
    /// returns 1 + the number of connections between root and the farthest node from it.
    /// # Example
    /// ```
    /// use rtrees::rbtree::*;
    /// #[derive(Copy, Clone)]
    /// struct PlaceHolder();
    /// impl Augment<PlaceHolder> for RBTree<u64, PlaceHolder, &'static str> {}
    /// type Tree = RBTree<u64, PlaceHolder,  &'static str>;
    /// let mut rbtree = Tree::new();
    /// assert_eq!(rbtree.get_level(), 0);
    /// for i in 0..1024 {
    ///     rbtree.insert(i, PlaceHolder(), "Random Value");
    /// }
    /// assert!(rbtree.get_level() >= 10 && rbtree.get_level() <= 20);
    /// ```
    pub fn get_level(&self) -> u64 {
        return if let Some(node) = self.as_ref() { node.get_level() } else { 0 };
    }

    pub(crate) fn sync_aug(&mut self) {
        self.as_mut().unwrap().sync_builtin_aug();
        self.sync_custom_aug();
    }

    fn balance(mut self) -> Self {
        self.sync_aug();
        if self.right_ref().is_red() && !self.left_ref().is_red() {
            self = self.unwrap().rotate_left().into();
        }
        if self.left_ref().is_red() && self.left_ref().left_ref().is_red() {
            self = self.unwrap().rotate_right().into();
        }
        if self.left_ref().is_red() && self.right_ref().is_red() {
            self.as_mut().unwrap().flip_colors();
        }
        return self;
    }

    fn insert_not_root(mut self, key: K, aug_data: A, data: V) -> RBTree<K, A, V> {
        if !self.is_node() {
            return Node::new(key, aug_data, data).into();
        }
        match key.cmp(&self.key()) {
            Ordering::Equal => self.set_data(data),
            Ordering::Greater => {
                let right = self.right();
                self.set_right(right.insert_not_root(key, aug_data, data));
            }
            Ordering::Less => {
                let left = self.left();
                self.set_left(left.insert_not_root(key, aug_data, data));
            }
        }
        self = self.balance();
        return self;
    }
    /// Deletes the minimum value in the tree and returns the data stored in that node.
    ///
    /// # Example
    /// ```
    /// use rtrees::rbtree::*;
    /// #[derive(Copy, Clone)]
    /// struct PlaceHolder();
    /// impl Augment<PlaceHolder> for RBTree<u64, PlaceHolder, &'static str> {}
    /// type Tree = RBTree<u64, PlaceHolder,  &'static str>;
    /// let mut rbtree = RBTree::new();
    /// rbtree.insert(0, PlaceHolder(), "First Insertion");
    /// rbtree.insert(5, PlaceHolder(), "Second Insertion");
    /// rbtree.insert(10, PlaceHolder(), "Third Insertion");
    /// assert_eq!(rbtree.delete_min().unwrap(), "First Insertion");
    /// assert_eq!(rbtree.delete_min().unwrap(), "Second Insertion");
    /// assert_eq!(rbtree.delete_min().unwrap(), "Third Insertion");
    /// assert_eq!(rbtree.delete_min(), None);
    /// ```
    pub fn delete_min(&mut self) -> Option<V> {
        if !self.is_node() {
            return None;
        }
        if !self.left_ref().is_red() && !self.right_ref().is_red() {
            self.as_mut().unwrap().color = COLOR::RED;
        }
        let mut result = self.take().delete_min_not_root();
        if result.0.is_node() {
            result.0.as_mut().unwrap().color = COLOR::BLACK;
        }
        *self = result.0;
        return result.1;
    }

    /// Inserts *data* associated with *key* into tree. *insert* does not support
    /// duplicated key. In case of inserting into an already existing key, the old
    /// *data* will silently be repalced by the new *data*.
    /// # Example
    /// ```
    /// use rtrees::rbtree::*;
    /// #[derive(Copy, Clone)]
    /// struct PlaceHolder();
    /// impl Augment<PlaceHolder> for RBTree<u64, PlaceHolder, u64> {}
    /// type Tree = RBTree<u64, PlaceHolder, u64>;
    /// let mut rbtree = Tree::new();
    /// rbtree.insert(0, PlaceHolder(), 0);
    /// rbtree.insert(1, PlaceHolder(), 1);
    /// rbtree.insert(2, PlaceHolder(), 2);
    /// assert_eq!(rbtree.search(0).unwrap(), &0);
    /// assert_eq!(rbtree.search(1).unwrap(), &1);
    /// assert_eq!(rbtree.search(2).unwrap(), &2);
    /// ```
    pub fn insert(&mut self, key: K, aug_data: A, data: V) {
        *self = self.take().insert_not_root(key, aug_data, data);
        self.as_mut().unwrap().color = COLOR::BLACK;
    }
    /// Force recalculating all agumented data from node matching *key* up to the root node.
    pub fn force_sync_aug(&mut self, key: K) {
        if !self.is_node() {
            return;
        }
        match key.cmp(&self.key()) {
            Ordering::Greater => self.right_mut().force_sync_aug(key),
            Ordering::Less => self.left_mut().force_sync_aug(key),
            _ => (),
        }
        self.sync_aug();
    }
    /// Returns a non mutable references of the data stored at *key*
    /// #example
    /// ```
    /// use rtrees::rbtree::*;
    /// #[derive(Copy, Clone)]
    /// struct PlaceHolder();
    /// impl Augment<PlaceHolder> for RBTree<u64, PlaceHolder, &'static str> {}
    /// type Tree = RBTree<u64, PlaceHolder,  &'static str>;
    /// let mut rbtree = Tree::new();
    /// rbtree.insert(0, PlaceHolder(), "First Insertion");
    /// rbtree.insert(5, PlaceHolder(), "Second Insertion");
    /// assert_eq!(rbtree.search(0).unwrap(), &"First Insertion");
    /// assert_eq!(rbtree.search(5).unwrap(), &"Second Insertion");
    /// assert_eq!(rbtree.search(21), None);
    /// ```
    pub fn search(&self, key: K) -> Option<&V> {
        let mut subtree = self;
        while subtree.is_node() {
            match key.cmp(&subtree.key()) {
                Ordering::Equal => return Some(subtree.data_ref()),
                Ordering::Greater => subtree = subtree.right_ref(),
                Ordering::Less => subtree = subtree.left_ref(),
            }
        }
        return None;
    }

    /// Returns a mutable references of the data stored at *key*.
    /// We assume that in mutable search caller might modify augmented data,
    /// #example
    /// ```
    /// use rtrees::rbtree::*;
    /// #[derive(Copy, Clone)]
    /// struct PlaceHolder();
    /// impl Augment<PlaceHolder> for RBTree<u64, PlaceHolder, String> {}
    /// type Tree = RBTree<u64, PlaceHolder,  String>;
    /// let mut rbtree = RBTree::new();
    /// rbtree.insert(0, PlaceHolder(), String::from("First Insertion"));
    /// rbtree.search_mut(0).unwrap().push_str(" Modified");
    /// assert_eq!(rbtree.search(0).unwrap(), &"First Insertion Modified");
    /// ```
    pub fn search_mut(&mut self, key: K) -> Option<&mut V> {
        let mut subtree = self;
        while subtree.is_node() {
            match key.cmp(&subtree.key()) {
                Ordering::Equal => return Some(subtree.data_mut()),
                Ordering::Greater => subtree = subtree.right_mut(),
                Ordering::Less => subtree = subtree.left_mut(),
            }
        }
        return None;
    }
    fn delete_random_node(mut self, key: K) -> (RBTree<K, A, V>, Option<V>) {
        if !self.is_node() {
            return (self, None);
        }
        let mut result;
        if key < self.key() {
            if !self.left_ref().is_red() && self.left_ref().is_node() && !self.left_ref().left_ref().is_red() {
                self = self.unwrap().move_red_left().into();
            }
            result = self.left().delete_random_node(key);
            self.set_left(result.0);
            result.0 = self;
        } else {
            if self.left_ref().is_red() {
                self = self.unwrap().rotate_right().into();
            }
            if key == self.key() && !self.right_ref().is_node() {
                return (RBTree::new(), Some(self.data()));
            }
            // do we really need self.right().is_node()
            if !self.right_ref().is_red() && self.right_ref().is_node() && !self.right_ref().left_ref().is_red() {
                self = self.unwrap().move_red_right().into();
            }
            if self.key() == key {
                let mut right = self.right();
                let x = right.node_min().as_mut().unwrap();
                self.as_mut().unwrap().key = x.key;
                self.as_mut().unwrap().aug_data = x.aug_data;
                mem::swap(&mut self.as_mut().unwrap().data, &mut x.data);
                result = right.delete_min_not_root();
                self.set_right(result.0);
                result.0 = self;
            } else {
                result = self.right().delete_random_node(key);
                self.set_right(result.0);
                result.0 = self;
            }
        }
        result.0 = result.0.balance();
        return result;
    }
    /// Deletes tree node represented by *key*. The return
    /// value is data stored there.
    ///
    /// # Example
    /// ```
    /// use rtrees::rbtree::*;
    /// #[derive(Copy, Clone)]
    /// struct PlaceHolder();
    /// impl Augment<PlaceHolder> for RBTree<u64, PlaceHolder, &'static str> {}
    /// type Tree = RBTree<u64, PlaceHolder,  &'static str>;
    /// let mut rbtree = Tree::new();
    /// rbtree.insert(10, PlaceHolder(), "First Insertion");
    /// rbtree.insert(15, PlaceHolder(), "Second Insertion");
    /// assert_eq!(rbtree.delete(10).unwrap(), "First Insertion");
    /// assert_eq!(rbtree.delete(15).unwrap(), "Second Insertion");
    /// ```

    pub fn delete(&mut self, key: K) -> Option<V> {
        if !self.left_ref().is_red() && !self.right_ref().is_red() {
            self.as_mut().unwrap().color = COLOR::RED;
        }
        let (tree, result) = self.take().delete_random_node(key);
        *self = tree;
        if self.is_node() {
            self.as_mut().unwrap().color = COLOR::BLACK;
        }
        return result;
    }
    fn node_min(&mut self) -> &mut RBTree<K, A, V> {
        if self.left_ref().is_node() {
            return self.left_mut().node_min();
        } else {
            return self;
        }
    }
    fn delete_min_not_root(mut self) -> (Self, Option<V>) {
        if !self.left_ref().is_node() {
            return (self.left(), Some(self.data()));
        }

        if !self.left_ref().is_red() && !self.left_ref().left_ref().is_red() {
            self = self.unwrap().move_red_left().into();
        }
        let (new_left, deleted_value) = self.left().delete_min_not_root();
        self.set_left(new_left);
        self = self.balance();
        return (self, deleted_value);
    }
}

impl<K: Ord + Copy, A: Copy, V> IntoIterator for RBTree<K, A, V>
where
    RBTree<K, A, V>: Augment<A>,
{
    type Item = (K, A, V);
    type IntoIter = TreeIterator<K, A, V>;
    fn into_iter(self) -> TreeIterator<K, A, V> {
        TreeIterator::new(self)
    }
}

impl<'a, K: Ord + Copy, A: Copy, V> IntoIterator for &'a RBTree<K, A, V>
where
    RBTree<K, A, V>: Augment<A>,
{
    type Item = (K, A, &'a V);
    type IntoIter = TreeRefIterator<'a, K, A, V>;
    fn into_iter(self) -> TreeRefIterator<'a, K, A, V> {
        TreeRefIterator::new(self)
    }
}

#[cfg(test)]
mod rbtree_tests {
    use super::{Augment, RBTree};
    #[derive(Copy, Clone)]
    struct PlaceHolder();
    impl Augment<PlaceHolder> for RBTree<u64, PlaceHolder, u64> {}
    type Tree = RBTree<u64, PlaceHolder, u64>;
    fn dfs(tree: &Tree, counter: &mut u64, blackcounter: u64, blackvec: &mut Vec<u64>) -> u64 {
        if !tree.is_node() {
            assert_eq!(tree.is_red(), false);
            blackvec.push(blackcounter + 1);
            return 0;
        }
        let newbc;
        if tree.is_red() {
            assert_eq!(tree.left_ref().is_red(), false);
            assert_eq!(tree.right_ref().is_red(), false);
            newbc = blackcounter;
        } else {
            newbc = blackcounter + 1
        }
        let l1 = dfs(tree.left_ref(), counter, newbc, blackvec);
        assert_eq!(tree.data_ref(), &tree.key());
        *counter = *counter + 1;
        let l2 = dfs(tree.right_ref(), counter, newbc, blackvec);
        return 1 + std::cmp::max(l1, l2);
    }
    fn verify_tree(tree: &Tree) {
        let mut counter = 0;
        let mut blackvec = Vec::with_capacity(tree.size() as usize / 2);
        let level = dfs(tree, &mut counter, 0, &mut blackvec);
        blackvec.sort();
        assert_eq!(tree.is_red(), false);
        assert_eq!(counter, tree.size());
        assert_eq!(level, tree.get_level());
        assert_eq!(blackvec[0], blackvec[blackvec.len() - 1]);
    }
    #[test]
    fn test_insert() {
        let mut rbtree = RBTree::new();
        assert_eq!(rbtree.get_level(), 0);
        for i in 0..1024 {
            rbtree.insert(i, PlaceHolder(), i);
            verify_tree(&rbtree);
        }
        assert!(rbtree.get_level() >= 10 && rbtree.get_level() <= 20);
        rbtree = RBTree::new();
        for i in (0..1024).rev() {
            rbtree.insert(i, PlaceHolder(), i);
            verify_tree(&rbtree);
        }
        assert!(rbtree.get_level() >= 10 && rbtree.get_level() <= 20);
    }

    #[test]
    fn test_search() {
        let mut rbtree = RBTree::new();
        assert_eq!(rbtree.get_level(), 0);
        for i in 0..1024 {
            rbtree.insert(i, PlaceHolder(), i);
        }
        for i in 0..1024 {
            assert_eq!(rbtree.search(i).unwrap(), &i);
        }
        assert_eq!(rbtree.search(1024), None);
    }
    #[test]
    fn test_delete() {
        let mut rbtree = RBTree::new();
        for i in 0..10 {
            rbtree.insert(i, PlaceHolder(), i);
            verify_tree(&rbtree);
        }
        for i in 0..10 {
            assert_eq!(rbtree.delete(i).unwrap(), i);
            verify_tree(&rbtree);
        }
        for i in 0..20 {
            rbtree.insert(i, PlaceHolder(), i);
            verify_tree(&rbtree);
        }
        for i in 15..20 {
            assert_eq!(rbtree.delete(i).unwrap(), i);
            verify_tree(&rbtree);
        }
        for i in 15..2000 {
            rbtree.insert(i, PlaceHolder(), i);
            verify_tree(&rbtree);
        }
        for i in 100..2000 {
            assert_eq!(rbtree.delete(i).unwrap(), i);
            verify_tree(&rbtree);
        }
        assert_eq!(rbtree.delete(500), None);
    }

    #[test]
    fn test_delete_min() {
        let mut rbtree = RBTree::new();
        assert_eq!(rbtree.delete_min(), None);
        for i in 0..2000 {
            rbtree.insert(i, PlaceHolder(), i);
        }
        for i in 0..2000 {
            assert_eq!(rbtree.delete_min().unwrap(), i);
        }
        assert_eq!(rbtree.delete_min(), None);
    }

    #[test]
    fn test_iter() {
        let mut rbtree = RBTree::new();
        for i in 0..2000 {
            rbtree.insert(i, PlaceHolder(), i);
        }
        let mut iter = rbtree.into_iter();
        for i in 0..2000 {
            assert_eq!(i, iter.next().unwrap().0)
        }
    }
    #[test]
    fn test_iter_ref() {
        let mut rbtree = RBTree::new();
        for i in 0..2000 {
            rbtree.insert(i, PlaceHolder(), i);
        }
        let mut iter = (&rbtree).into_iter();
        for i in 0..2000 {
            assert_eq!(i, iter.next().unwrap().0)
        }
    }

    #[test]
    fn test_iter_empty() {
        // I am fully aware that the assers will never be evaluated.
        // The point here is that at some point in time, iterating over
        // empty tree, triggered a bug that crashed.
        let rbtree: RBTree<u64, PlaceHolder, u64> = RBTree::new();
        for (key, _, value) in rbtree.into_iter() {
            assert_eq!(key, 5);
            assert_eq!(value, 5);
        }
    }
    #[test]
    fn test_iter_ref_empty() {
        // I am fully aware that the assers will never be evaluated.
        // The point here is that at some point in time, iterating over
        // empty tree, triggered a bug that crashed.
        let rbtree: RBTree<u64, PlaceHolder, u64> = RBTree::new();
        for (key, _, value) in (&rbtree).into_iter() {
            assert_eq!(key, 5);
            assert_eq!(*value, 5);
        }
    }
}
