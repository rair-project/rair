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

pub(super) type RBTreeOp<K, V> = Option<Box<Node<K, V>>>;

#[derive(Default)]
pub struct RBTree<K: Ord + Copy, V>(RBTreeOp<K, V>);

impl<K: Ord + Copy, V> From<Node<K, V>> for RBTree<K, V> {
    fn from(node: Node<K, V>) -> Self {
        RBTree(Some(Box::new(node)))
    }
}

impl<K: Ord + Copy, V> RBTree<K, V> {
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

    pub(super) fn insert_random_node(self, key: K, data: V) -> RBTree<K, V> {
        return RBTree::from(match self.0 {
            Some(root) => root.insert(key, data),
            None => Node::new(key, data),
        });
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
    pub(super) fn delete_min_not_root(mut self) -> (Self, Option<V>) {
        if !self.left_ref().is_node() {
            return (self.left(), Some(self.data()));
        }
        return self.unwrap().delete_min_not_root();
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
        *self = self.take().insert_random_node(key, data);
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
            if key == subtree.key() {
                return Some(subtree.data_ref());
            } else if key < subtree.key() {
                subtree = subtree.left_ref();
            } else {
                subtree = subtree.right_ref()
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
            if key == subtree.key() {
                return Some(subtree.data_mut());
            } else if key < subtree.key() {
                subtree = subtree.left_mut();
            } else {
                subtree = subtree.right_mut()
            }
        }
        return None;
    }
    pub(super) fn delete_random_node(self, key: K) -> (RBTree<K, V>, Option<V>) {
        if self.is_node() {
            let (node, data) = self.unwrap().delete(key);
            return (node, data);
        } else {
            return (self, None);
        }
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
        if self.search(key).is_none() {
            return None;
        }
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
    // Implementing wrapper for Option functionality
    #[inline]
    pub(crate) fn take(&mut self) -> RBTree<K, V> {
        RBTree(self.0.take())
    }

    #[inline]
    pub(crate) fn is_node(&self) -> bool {
        self.0.is_some()
    }

    #[inline]
    pub(crate) fn as_mut(&mut self) -> Option<&mut Box<Node<K, V>>> {
        self.0.as_mut()
    }

    #[inline]
    pub(crate) fn as_ref(&self) -> Option<&Box<Node<K, V>>> {
        self.0.as_ref()
    }

    #[inline]
    pub(crate) fn unwrap(self) -> Box<Node<K, V>> {
        self.0.unwrap()
    }
    #[inline]
    pub(crate) fn is_red(&self) -> bool {
        self.is_node() && self.as_ref().unwrap().is_red()
    }
    /// Returns copy of key of the current Tree node
    /// # Panics
    /// panics if current subtree is not a *node*
    pub fn key(&self) -> K {
        self.as_ref().unwrap().key
    }
    /// Returns data stored in the current Tree node
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
