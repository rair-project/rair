//! Iterator implementation for ist.

use super::interval::Interval;
use super::rb_helpers::AugData;
use super::tree::IST;
use crate::rbtree::TreeIterator;
use alloc::vec::IntoIter;

/// Iterator for [IST].
pub struct ISTIterator<K: Ord + Copy, V> {
    current_iter: IntoIter<V>,
    hi: Option<K>,
    lo: Option<K>,
    tree_iter: TreeIterator<Interval<K>, AugData<K>, Vec<V>>,
}

impl<K: Ord + Copy, V> ISTIterator<K, V> {
    pub(crate) fn new(root: IST<K, V>) -> ISTIterator<K, V> {
        ISTIterator {
            current_iter: Vec::new().into_iter(),
            hi: None,
            lo: None,
            tree_iter: root.root.into_iter(),
        }
    }
}
impl<K: Ord + Copy, V> Iterator for ISTIterator<K, V> {
    type Item = (K, K, V);

    fn next(&mut self) -> Option<(K, K, V)> {
        if let Some(data) = self.current_iter.next() {
            return Some((self.lo.unwrap(), self.hi.unwrap(), data));
        }
        if let Some((k, _, v)) = self.tree_iter.next() {
            self.current_iter = v.into_iter();
            self.lo = Some(k.lo);
            self.hi = Some(k.hi);
        } else {
            return None;
        }
        Some((
            self.lo.unwrap(),
            self.hi.unwrap(),
            self.current_iter.next().unwrap(),
        ))
    }
}
