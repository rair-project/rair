//! Copy of iter.rs except for it deals with references :(.

use super::rbtree_wrapper::{Augment, RBTree};

// In case of iter we would tear down the tree structure and consumed nodes will no
// longer exist but here we need to mark if a node is ever traversed or not and that
// is precisely what this enum is for.
enum Hint<'a, K: Ord + Copy, A: Copy, V> {
    // means check only this node.
    NA(&'a RBTree<K, A, V>),
    // means check left, right and data of this node.
    LR(&'a RBTree<K, A, V>),
}

/// Iterator for [RBTree] reference
pub struct TreeRefIterator<'a, K: Ord + Copy, A: Copy, V> {
    right: Vec<Hint<'a, K, A, V>>,
    current: Option<&'a RBTree<K, A, V>>,
}

impl<'a, K: Ord + Copy, A: Copy, V> TreeRefIterator<'a, K, A, V>
where
    RBTree<K, A, V>: Augment<A>,
{
    pub(crate) fn new(root: &'a RBTree<K, A, V>) -> TreeRefIterator<'a, K, A, V> {
        let mut iter = TreeRefIterator {
            right: vec![],
            current: None,
        };
        iter.add_subtree(root);
        iter
    }
    fn add_subtree(&mut self, root: &'a RBTree<K, A, V>) {
        let mut node = root;
        while node.is_node() {
            if node.right_ref().is_node() {
                self.right.push(Hint::LR(node.right_ref()));
            }
            if node.left_ref().is_node() {
                let tmp = node.left_ref();
                self.right.push(Hint::NA(node));
                node = tmp;
            } else {
                break;
            }
        }
        self.current = if node.is_node() { Some(node) } else { None }
    }
}
impl<'a, K: Ord + Copy, A: Copy, V> Iterator for TreeRefIterator<'a, K, A, V>
where
    RBTree<K, A, V>: Augment<A>,
{
    type Item = (K, A, &'a V);

    fn next(&mut self) -> Option<(K, A, &'a V)> {
        let result;
        if let Some(node) = self.current.take() {
            result = Some((node.key(), node.aug_data(), node.data_ref()));
        } else {
            return None;
        }
        if let Some(node) = self.right.pop() {
            match node {
                Hint::NA(node) => self.current = Some(node),
                Hint::LR(node) => self.add_subtree(node),
            }
        }
        result
    }
}
