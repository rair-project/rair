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

/// Generic BK-Tree Template used to store dictionary like
/// structures and perform fuzzy search on them. [K] must implement trait
/// [Distance] before it can be used as key here.
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

        return (exact, close);
    }
}
/// This trait used by [BKTree] to tell how close are 2 objects when fuzzy searching.
/// In case of strings, the [distance] function could be something like Levenshtein distance,
/// Damerau–Levenshtein distance, Optimal string alignment distance or anything similar.
pub trait Distance {
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
        return if let Some(root) = &self.root { root.find(&key, tolerance) } else { (Vec::new(), Vec::new()) };
    }
}

fn osa_distance(str1: &str, str2: &str) -> u64 {
    // Optimal string alignment distance
    if str1 == str2 {
        return 0;
    }
    let a = str1.as_bytes();
    let b = str2.as_bytes();
    let mut d = vec![vec![0; b.len() + 1]; a.len() + 1];
    for (i, item) in d.iter_mut().enumerate().take(a.len() + 1) {
        item[0] = i as u64;
    }
    for (j, item) in d[0].iter_mut().enumerate().take(b.len() + 1) {
        *item = j as u64;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            d[i][j] = min(
                d[i - 1][j] + 1, // deletion
                min(
                    d[i][j - 1] + 1, // insertion
                    d[i - 1][j - 1] + cost,
                ),
            ); // substitution
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = min(d[i][j], d[i - 2][j - 2] + cost) // transposition
            }
        }
    }
    return d[a.len()][b.len()];
}

impl Distance for String {
    fn distance(&self, other: &Self) -> u64 {
        osa_distance(self, other)
    }
}

/// a BKTree with string based Key and [Distance] trait optimized for
/// capturing spelling and typing mistakes.
///
/// # Example
/// ```
/// use rtrees::bktree::SpellTree;
/// let mut tree :SpellTree<&str> = SpellTree::new();
/// tree.insert("hello".to_string(), &"hello");
/// tree.insert("hell".to_string(), "&hell");
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
            ("helo wolrd", "hello world", 2),
            ("open", "opnre", 3), // In case of demere Lavenstien distance this might have been 2
            ("CA", "ABC", 3),
        ];
        for (s1, s2, d) in s.iter() {
            assert_eq!(osa_distance(s1, s2), *d);
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
        res = tree.find(&"helicoptre".to_string(), 1);
        assert_eq!(res.0.len(), 0);
        assert_eq!(res.1.len(), 1);
        assert_eq!(res.1[0], "helicopter");
        res = tree.find(&"attempt".to_string(), 1);
        assert_eq!(res.0.len(), 0);
        assert_eq!(res.1.len(), 0);
    }
}
