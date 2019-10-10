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
use super::node::*;
use std::cmp::Ordering;
use std::mem;

pub(super) type RBTreeOp<K, V> = Option<Box<Node<K, V>>>;

#[derive(Default)]
pub struct RBTree<K: Ord + Copy, V>(RBTreeOp<K, V>);

impl<K: Ord + Copy, V> From<Node<K, V>> for RBTree<K, V> {
    fn from(node: Node<K, V>) -> Self {
        RBTree(Some(Box::new(node)))
    }
}

impl<K: Ord + Copy, V> RBTree<K, V> {
    // Implementing wrapper for Option functionality
    #[inline]
    pub(super) fn take(&mut self) -> RBTree<K, V> {
        RBTree(self.0.take())
    }

    #[inline]
    pub(super) fn is_node(&self) -> bool {
        self.0.is_some()
    }

    #[inline]
    pub(super) fn as_mut(&mut self) -> Option<&mut Node<K, V>> {
        self.0.as_mut().map(|x| &mut **x)
    }

    #[inline]
    pub(super) fn as_ref(&self) -> Option<&Node<K, V>> {
        self.0.as_ref().map(|x| &**x)
    }

    #[inline]
    pub(super) fn unwrap(self) -> Box<Node<K, V>> {
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
    pub fn set_left(&mut self, subtree: RBTree<K, V>) {
        self.as_mut().unwrap().left = subtree;
    }

    /// Returns the left subtree after ripping it from the current node.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn left(&mut self) -> RBTree<K, V> {
        self.as_mut().unwrap().left.take()
    }

    /// Returns a non-mutable reference to left subtree.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn left_ref(&self) -> &RBTree<K, V> {
        &self.as_ref().unwrap().left
    }

    /// Returns a mutable reference to left subtree.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn left_mut(&mut self) -> &mut RBTree<K, V> {
        &mut self.as_mut().unwrap().left
    }

    /// Set the right subtree of the current Node.
    /// # Panics
    /// panics if current subtree is not a *node*.
    pub fn set_right(&mut self, subtree: RBTree<K, V>) {
        self.as_mut().unwrap().right = subtree;
    }

    /// Returns the right subtree after ripping it from the current node.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn right(&mut self) -> RBTree<K, V> {
        self.as_mut().unwrap().right.take()
    }

    /// Returns a non-mutable reference to right subtree.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn right_ref(&self) -> &RBTree<K, V> {
        &self.as_ref().unwrap().right
    }

    /// Returns a mutable reference to right subtree.
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn right_mut(&mut self) -> &mut RBTree<K, V> {
        &mut self.as_mut().unwrap().right
    }

    /// Returns new Red Black Tree
    /// # Example
    /// ```
    /// use rtrees::rbtree::RBTree;
    /// let mut rbtree: RBTree<u64, &'static str> = RBTree::new();
    /// ```
    pub fn new() -> RBTree<K, V> {
        RBTree(None)
    }

    /// Returns the number of elements in the tree
    /// # Example
    /// ```
    /// use rtrees::rbtree::RBTree;
    /// let mut rbtree: RBTree<u64, &'static str> = RBTree::new();
    /// assert_eq!(rbtree.size(), 0);
    /// rbtree.insert(0,"Zero");
    /// assert_eq!(rbtree.size(), 1);
    /// rbtree.insert(1, "One");
    /// assert_eq!(rbtree.size(), 2);
    /// rbtree.insert(2, "Two");
    /// assert_eq!(rbtree.size(), 3);
    /// ```
    pub fn size(&self) -> u64 {
        match &self.0 {
            Some(node) => return node.size(),
            None => return 0,
        }
    }

    /// 0 will be returned in case of empty tree. If tree has nodes, then *get_level*
    /// returns 1 + the number of connections between root and the farthest node from it.
    /// # Example
    /// ```
    /// use rtrees::rbtree::RBTree;
    /// let mut rbtree: RBTree<u64, &'static str> = RBTree::new();
    /// assert_eq!(rbtree.get_level(), 0);
    /// for i in 0..1024 {
    ///     rbtree.insert(i, "Random Value");
    /// }
    /// assert!(rbtree.get_level() >= 10 && rbtree.get_level() <= 20);
    /// ```
    pub fn get_level(&self) -> u64 {
        match self.as_ref() {
            Some(node) => return node.get_level(),
            None => return 0,
        }
    }
    fn sync_aug(&mut self) {
        self.as_mut().unwrap().sync_builtin_aug();
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
    fn insert_not_root(mut self, key: K, data: V) -> RBTree<K, V> {
        if !self.is_node() {
            return Node::new(key, data).into();
        }
        match key.cmp(&self.key()) {
            Ordering::Equal => self.set_data(data),
            Ordering::Greater => {
                let right = self.right();
                self.set_right(right.insert_not_root(key, data));
            }
            Ordering::Less => {
                let left = self.left();
                self.set_left(left.insert_not_root(key, data));
            }
        }
        self = self.balance();
        return self;
    }
    /// Deletes the minimum value in the tree and returns the data stored in that node.
    /// #example
    /// ```
    /// use rtrees::rbtree::RBTree;
    /// let mut rbtree = RBTree::new();
    /// rbtree.insert(0, "First Insertion");
    /// rbtree.insert(5, "Second Insertion");
    /// rbtree.insert(10, "Third Insertion");
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
    /// use rtrees::rbtree::RBTree;
    /// let mut rbtree: RBTree<u64, &'static str> = RBTree::new();
    /// /*rbtree.insert(0,10, "First Insertion");
    /// assert_eq!(rbtree.at(0), [&"First Insertion"]);
    /// assert_eq!(rbtree.at(2), [&"First Insertion"]);
    /// assert_eq!(rbtree.at(10), [&"First Insertion"]);*/
    /// ```
    pub fn insert(&mut self, key: K, data: V) {
        *self = self.take().insert_not_root(key, data);
        self.as_mut().unwrap().color = COLOR::BLACK;
    }
    /// Returns a non mutable references of the data stored at *key*
    /// #example
    /// ```
    /// use rtrees::rbtree::RBTree;
    /// let mut rbtree = RBTree::new();
    /// rbtree.insert(0, "First Insertion");
    /// rbtree.insert(5, "Second Insertion");
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
    /// #example
    /// ```
    /// use rtrees::rbtree::RBTree;
    /// let mut rbtree = RBTree::new();
    /// rbtree.insert(0, String::from("First Insertion"));
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
    pub(super) fn delete_random_node(mut self, key: K) -> (RBTree<K, V>, Option<V>) {
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
    /// use rtrees::rbtree::RBTree;
    /// let mut rbtree = RBTree::new();
    /// rbtree.insert(10, String::from("First Insertion"));
    /// rbtree.insert(15, String::from("Second Insertion"));
    /// assert_eq!(rbtree.delete(10).unwrap(), "First Insertion");
    /// assert_eq!(rbtree.delete(15).unwrap(), "Second Insertion");
    /// ```

    pub fn delete(&mut self, key: K) -> Option<V> {
        self.search(key)?; // Do we really need this line
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
    fn node_min(&mut self) -> &mut RBTree<K, V> {
        if self.left_ref().is_node() {
            return self.left_mut().node_min();
        } else {
            return self;
        }
    }
    pub(super) fn delete_min_not_root(mut self) -> (Self, Option<V>) {
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
#[cfg(test)]
mod rbtree_tests {
    use super::RBTree;
    fn dfs(tree: &RBTree<u64, u64>, counter: &mut u64, blackcounter: u64, blackvec: &mut Vec<u64>) -> u64 {
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
    fn verify_tree(tree: &RBTree<u64, u64>) {
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
            rbtree.insert(i, i);
            verify_tree(&rbtree);
        }
        assert!(rbtree.get_level() >= 10 && rbtree.get_level() <= 20);
        rbtree = RBTree::new();
        for i in (0..1024).rev() {
            rbtree.insert(i, i);
            verify_tree(&rbtree);
        }
        assert!(rbtree.get_level() >= 10 && rbtree.get_level() <= 20);
    }

    #[test]
    fn test_search() {
        let mut rbtree = RBTree::new();
        assert_eq!(rbtree.get_level(), 0);
        for i in 0..1024 {
            rbtree.insert(i, i);
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
            rbtree.insert(i, i);
            verify_tree(&rbtree);
        }
        for i in 0..10 {
            assert_eq!(rbtree.delete(i).unwrap(), i);
            verify_tree(&rbtree);
        }
        for i in 0..20 {
            rbtree.insert(i, i);
            verify_tree(&rbtree);
        }
        for i in 15..20 {
            assert_eq!(rbtree.delete(i).unwrap(), i);
            verify_tree(&rbtree);
        }
        for i in 15..2000 {
            rbtree.insert(i, i);
            verify_tree(&rbtree);
        }
        for i in 100..2000 {
            assert_eq!(rbtree.delete(i).unwrap(), i);
            verify_tree(&rbtree);
        }
    }
}
