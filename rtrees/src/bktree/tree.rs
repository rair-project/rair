/*
 * bktree: Approximate String search data structure.
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
use std::cmp::min;
use std::collections::HashMap;

// SIMD-accelerated edit distance routines
extern crate triple_accel;
use self::triple_accel::levenshtein_exp;

/// Generic BK-Tree Template used to store dictionary like
/// structures and perform fuzzy search on them. *K* must implement trait
/// distance before it can be used as key here.
#[derive(Default)]
pub struct BKTree<K, V>
where
    K: Distance,
{
    root: Option<BKTreeNode<K, V>>,
}
struct BKTreeNode<K, V>
where
    K: Distance,
{
    key: K,
    value: V,
    children: HashMap<u64, BKTreeNode<K, V>>,
}
impl<K, V> BKTreeNode<K, V>
where
    K: Distance,
{
    fn new(key: K, value: V) -> Self {
        BKTreeNode { key, value, children: HashMap::new() }
    }
    fn insert(&mut self, key: K, value: V) {
        let distance = self.key.distance(&key);
        if let Some(child) = self.children.get_mut(&distance) {
            child.insert(key, value);
        } else {
            self.children.insert(distance, BKTreeNode::new(key, value));
        }
    }

    fn find(&self, key: &K, tolerance: u64) -> (Vec<&V>, Vec<&K>) {
        let (mut exact, mut close) = (Vec::new(), Vec::new());
        let current_distance = self.key.distance(&key);
        if current_distance == 0 {
            exact.push(&self.value);
        } else if current_distance <= tolerance {
            close.push(&self.key);
        }
        for i in current_distance.saturating_sub(tolerance)..=current_distance.saturating_add(tolerance) {
            if let Some(child) = self.children.get(&i) {
                let mut result = child.find(key, tolerance);
                exact.append(&mut result.0);
                close.append(&mut result.1);
            }
        }

        (exact, close)
    }
}
/// This trait used by [BKTree] to tell how close are 2 objects when fuzzy searching.
/// In case of strings, the distance function could be something like Levenshtein distance,
/// Damerau–Levenshtein distance, Optimal string alignment distance or anything similar.
pub trait Distance {
    /// Calculate the distance between two nodes in the [BKTree]
    fn distance(&self, other: &Self) -> u64;
}

impl<K, V> BKTree<K, V>
where
    K: Distance,
{
    /// Returns a new BK-Tree
    pub fn new() -> BKTree<K, V> {
        BKTree { root: None }
    }

    /// Inserts a new (*key*, *value*) pair into the KB-Tree
    pub fn insert(&mut self, key: K, value: V) {
        if let Some(root) = &mut self.root {
            root.insert(key, value);
        } else {
            self.root = Some(BKTreeNode::new(key, value));
        }
    }

    /// Search for the closest Item to *key* with a *tolerance* factor.
    /// The return value is tuple of 2 vectors, the first of exact matches
    /// and the second is are approximate matches.
    ///
    /// Two keys *key1* and *key2* are said to be approximate match IFF
    /// `key1.distance(key2) <= tolerance`.
    pub fn find(&self, key: &K, tolerance: u64) -> (Vec<&V>, Vec<&K>) {
        if let Some(root) = &self.root {
            root.find(&key, tolerance)
        } else {
            (Vec::new(), Vec::new())
        }
    }
}

impl Distance for String {
    fn distance(&self, other: &Self) -> u64 {
        levenshtein_exp(self.as_bytes(), other.as_bytes()) as u64
    }
}

/// A BKTree with string based Key and distance trait optimized for
/// capturing spelling and typing mistakes.
///
/// By default, this uses Levenshtein distance accelerated with SIMD.
///
/// # Example
/// ```
/// use rtrees::bktree::SpellTree;
/// let mut tree :SpellTree<&str> = SpellTree::new();
/// tree.insert("hello".to_string(), &"hello");
/// tree.insert("hell".to_string(), &"hell");
/// tree.insert("help".to_string(), &"help");
/// tree.insert("boy".to_string(), &"boy");
/// tree.insert("interaction".to_string(), &"interaction");
/// tree.insert("mistake".to_string(), &"mistake");
/// let (exact, approx) = tree.find(&"hello".to_string(), 1);
/// //assert_eq!(exact[0], "hello");
/// ```
pub type SpellTree<V> = BKTree<String, V>;

#[cfg(test)]
mod bktree_tests {
    use super::*;
    #[test]
    fn test_dl_distance() {
        let s = [
            ("hello world", "hello world", 0),
            ("hello world", "hello world ", 1),
            ("hello world", "h ello World", 2),
        ];
        for (s1, s2, d) in s.iter() {
            assert_eq!(levenshtein_exp(s1.as_bytes(), s2.as_bytes()) as u64, *d);
        }
    }
    #[test]
    fn test_spell_tree_one_level() {
        let mut tree: SpellTree<&str> = SpellTree::new();
        let words = ["hello", "hell", "held", "helicopter", "helium", "helix", "helmet"];
        for word in words.iter() {
            tree.insert(word.to_string(), word);
        }
        let mut res = tree.find(&"hello".to_string(), 1);
        assert_eq!(res.0[0], &"hello");
        assert_eq!(res.1.len(), 1);
        assert_eq!(res.1[0], &"hell");
        res = tree.find(&"helicoptrr".to_string(), 1);
        assert_eq!(res.0.len(), 0);
        assert_eq!(res.1.len(), 1);
        assert_eq!(res.1[0], "helicopter");
        res = tree.find(&"attempt".to_string(), 1);
        assert_eq!(res.0.len(), 0);
        assert_eq!(res.1.len(), 0);
    }
}
