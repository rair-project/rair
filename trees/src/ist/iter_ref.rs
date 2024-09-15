//! similar to iter but it iterates over reference instead of consuming the tree.

use super::interval::Interval;
use super::rb_helpers::AugData;
use super::tree::IST;
use crate::rbtree::TreeRefIterator;
use core::slice::Iter;
/// Iterator for [IST] reference
pub struct ISTRefIterator<'a, K: Ord + Copy, V> {
    tree_iter: TreeRefIterator<'a, Interval<K>, AugData<K>, Vec<V>>,
    lo: Option<K>,
    hi: Option<K>,
    current_iter: Iter<'a, V>,
}

impl<'a, K: Ord + Copy, V> ISTRefIterator<'a, K, V> {
    pub(crate) fn new(root: &'a IST<K, V>) -> ISTRefIterator<'_, K, V> {
        ISTRefIterator {
            tree_iter: (&root.root).into_iter(),
            lo: None,
            hi: None,
            current_iter: [].iter(),
        }
    }
}
impl<'a, K: Ord + Copy, V> Iterator for ISTRefIterator<'a, K, V> {
    type Item = (K, K, &'a V);

    fn next(&mut self) -> Option<(K, K, &'a V)> {
        if let Some(data) = self.current_iter.next() {
            return Some((self.lo.unwrap(), self.hi.unwrap(), data));
        }
        if let Some((k, _, v)) = self.tree_iter.next() {
            self.current_iter = v.iter();
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
