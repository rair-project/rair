/**
 * serialize.rs: Serialize and deserialize interval search tree.
 *  Copyright (C) 2019  Oddcoder
 *
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
 **/
use super::tree::*;
use serde::de;
use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::fmt;
impl<K: Ord + Copy + Serialize, V: Serialize> Serialize for IST<K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        // Size hint
        seq.serialize_element(&self.size())?;
        for (lo, hi, data) in self {
            seq.serialize_element(&lo)?;
            seq.serialize_element(&hi)?;
            seq.serialize_element(data)?;
        }
        seq.end()
    }
}

struct ISTVisitor<K, V>(Option<(K, V)>);

impl<'de, K, V> Visitor<'de> for ISTVisitor<K, V>
where
    K: Ord + Copy + Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = IST<K, V>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("struct IST")
    }
    fn visit_seq<VI>(self, mut seq: VI) -> Result<IST<K, V>, VI::Error>
    where
        VI: SeqAccess<'de>,
    {
        // size must be there
        let size: u64 = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let mut tree = IST::new();
        for _ in 0..size {
            let lo = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let hi = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let data = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            tree.insert(lo, hi, data);
        }
        Ok(tree)
    }
}

impl<'de, K, V> Deserialize<'de> for IST<K, V>
where
    K: Ord + Copy + Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = ISTVisitor(None);
        deserializer.deserialize_seq(visitor)
    }
}

#[cfg(test)]
mod test_ist_serialize {
    use super::*;
    #[test]
    fn test_serialize() {
        let mut ist: IST<u64, &'static str> = IST::new();
        ist.insert(50, 60, "[50, 60]");
        ist.insert(20, 30, "[20, 30]");
        ist.insert(80, 90, "[80, 90]");
        ist.insert(10, 100, "[10, 100]");
        ist.insert(30, 40, "[30, 40]");
        ist.insert(65, 70, "[65, 70]");
        ist.insert(85, 95, "[85, 95]");
        ist.insert(25, 35, "[25, 35]");
        ist.insert(66, 200, "[66, 200]");
        ist.insert(50, 60, "Attempt2");
        let serialized = serde_json::to_string(&ist).unwrap();
        let deserialized: IST<u64, &str> = serde_json::from_str(&serialized).unwrap();
        let mut iter = deserialized.into_iter();
        assert_eq!(iter.next().unwrap(), (10, 100, "[10, 100]"));
        assert_eq!(iter.next().unwrap(), (20, 30, "[20, 30]"));
        assert_eq!(iter.next().unwrap(), (25, 35, "[25, 35]"));
        assert_eq!(iter.next().unwrap(), (30, 40, "[30, 40]"));
        assert_eq!(iter.next().unwrap(), (50, 60, "[50, 60]"));
        assert_eq!(iter.next().unwrap(), (50, 60, "Attempt2"));
        assert_eq!(iter.next().unwrap(), (65, 70, "[65, 70]"));
        assert_eq!(iter.next().unwrap(), (66, 200, "[66, 200]"));
        assert_eq!(iter.next().unwrap(), (80, 90, "[80, 90]"));
        assert_eq!(iter.next().unwrap(), (85, 95, "[85, 95]"));
    }
}
