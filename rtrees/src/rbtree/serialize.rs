/*
 * serialize.rs: Serializing  RBTree.
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
use super::rbtree_wrapper::{Augment, RBTree};
use serde::de;
use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::fmt;

impl<K: Ord + Copy + Serialize, A: Copy + Serialize, V: Serialize> Serialize for RBTree<K, A, V>
where
    RBTree<K, A, V>: Augment<A>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        // Size hint
        seq.serialize_element(&self.size())?;
        for (key, aug_data, data) in self {
            seq.serialize_element(&key)?;
            seq.serialize_element(&data)?;
            seq.serialize_element(&aug_data)?;
        }
        seq.end()
    }
}
struct RBTreeVisitor<K, A, V>(Option<(K, A, V)>);
impl<'de, K, A, V> Visitor<'de> for RBTreeVisitor<K, A, V>
where
    RBTree<K, A, V>: Augment<A>,
    K: Ord + Copy + Deserialize<'de>,
    A: Copy + Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = RBTree<K, A, V>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct RBTree")
    }
    fn visit_seq<VI>(self, mut seq: VI) -> Result<RBTree<K, A, V>, VI::Error>
    where
        VI: SeqAccess<'de>,
    {
        // size must be there
        let size: u64 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let mut tree = RBTree::new();
        for _ in 0..size {
            let key = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let data = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let aug_data = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
            tree.insert(key, aug_data, data);
        }
        return Ok(tree);
    }
}

impl<'de, K, A, V> Deserialize<'de> for RBTree<K, A, V>
where
    RBTree<K, A, V>: Augment<A>,
    K: Ord + Copy + Deserialize<'de>,
    A: Copy + Deserialize<'de>,
    V: Copy + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = RBTreeVisitor(None);
        deserializer.deserialize_seq(visitor)
    }
}
#[cfg(test)]
mod test_rb_tree_serializing {
    use super::*;
    use serde::{Deserialize, Serialize};
    #[derive(Copy, Clone, Serialize, Deserialize)]
    struct PlaceHolder();
    impl Augment<PlaceHolder> for RBTree<u64, PlaceHolder, u64> {}
    #[test]
    fn test_serde() {
        let mut rbtree = RBTree::new();
        for i in 0..100 {
            rbtree.insert(i, PlaceHolder(), i);
        }
        let serialized = serde_json::to_string(&rbtree).unwrap();
        let mut deserialized: RBTree<u64, PlaceHolder, u64> = serde_json::from_str(&serialized).unwrap();
        for i in 0..100 {
            assert_eq!(deserialized.delete_min().unwrap(), i);
        }
    }
}
