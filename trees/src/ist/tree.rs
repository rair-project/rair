/*
 * tree.rs: Augmented Interval Search Tree
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
 */
/*
 *  The plan is to make it self balancing tree
 *  implemented using  left-leaning red-black tree:
 *  https://www.cs.princeton.edu/~rs/talks/LLRB/LLRB.pdf
 */
use super::interval::Interval;
use super::iter::ISTIterator;
use super::iter_ref::ISTRefIterator;
use super::rb_helpers::{AugData, ISTHelpers};
use crate::rbtree::{Augment, RBTree};

/// Interval Query data type based on augmented binary search tree,
/// written as *IST* but pronounced 'Interval Search Tree'.
/// IST is balanced using [RBTree].
///
/// IST support handling overlapping intervals, non-overlapping intervals,
/// as well as keeping track of multiple insertions into same interval.
pub struct IST<K: Ord + Copy, V> {
    pub(super) root: RBTree<Interval<K>, AugData<K>, Vec<V>>,
}

impl<K: Ord + Copy, V> Default for IST<K, V> {
    fn default() -> Self {
        IST::new()
    }
}

impl<K: Ord + Copy, V> Augment<AugData<K>> for RBTree<Interval<K>, AugData<K>, Vec<V>> {
    fn sync_custom_aug(&mut self) {
        let mut aug_data = AugData::new(self.key(), self.data_ref().len() as u64);
        if self.left_ref().is_node() {
            let l = self.left_ref().aug_data();
            aug_data.interval.absorb(l.interval);
            aug_data.size += l.size;
        }
        if self.right_ref().is_node() {
            let r = self.right_ref().aug_data();
            aug_data.interval.absorb(r.interval);
            aug_data.size += r.size;
        }
        self.set_aug_data(aug_data);
    }
}
impl<K: Ord + Copy, V> IST<K, V> {
    /// Returns new Interval Search Tree
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist: IST<u64, &'static str> = IST::new();
    /// ```
    pub fn new() -> IST<K, V> {
        IST {
            root: RBTree::new(),
        }
    }

    /// Returns the number of elements in the IST
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist: IST<u64, &'static str> = IST::new();
    /// assert_eq!(ist.size(), 0);
    /// ist.insert(0, 5, &"[0, 5]");
    /// assert_eq!(ist.size(), 1);
    /// ist.insert(30, 34, &"[30, 34]");
    /// assert_eq!(ist.size(), 2);
    /// ist.insert(4, 11, &"[4, 11]");
    ///assert_eq!(ist.size(), 3);
    /// ```
    pub fn size(&self) -> u64 {
        if !self.root.is_node() {
            return 0;
        }
        self.root.aug_data().size
    }

    /// 0 will be returned in case of empty *IST*. If *IST* has nodes, then *get_level*
    /// returns 1 + the number of connections between root and the farthest node from it.
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist: IST<u64, &'static str> = IST::new();
    /// assert_eq!(ist.get_level(), 0);
    /// ist.insert(4, 11, &"[4, 11]");
    /// assert_eq!(ist.get_level(), 1);
    /// ist.insert(30, 34, &"[30, 34]");
    /// assert_eq!(ist.get_level(), 2);
    /// ist.insert(0, 5, &"[0, 5]");
    /// assert_eq!(ist.get_level(), 2);
    /// ist.insert(0, 3, &"[0, 3]");
    /// assert_eq!(ist.get_level(), 3);
    /// ```
    pub fn get_level(&self) -> u64 {
        self.root.get_level()
    }

    /// Inserts an *element* into closed interval *[ lo, hi ]*. Insertion guarantess
    /// <math xmlns="http://www.w3.org/1998/Math/MathML"><mrow><mi>&#x1D4AA;</mi>
    /// <mrow><mo form="prefix">(</mo><mi>log</mi><mi>n</mi><mo form="postfix">)
    /// </mo></mrow></mrow></math>
    /// time. Insertion supports inserting multiple time into the same interval,
    ///  and keeps track of all inserted data.
    /// # Panics
    /// Panics if *lo* > *hi*
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(0,10, "First Insertion");
    ///assert_eq!(ist.at(0), [&"First Insertion"]);
    /// assert_eq!(ist.at(2), [&"First Insertion"]);
    /// assert_eq!(ist.at(10), [&"First Insertion"]);
    /// ```
    pub fn insert(&mut self, lo: K, hi: K, data: V) {
        assert!(lo <= hi);
        let interval = Interval::new(lo, hi);
        let aug_data = AugData::new(interval, 1);
        if let Some(data_vec) = self.root.search_mut(interval) {
            data_vec.push(data);
            self.root.force_sync_aug(interval);
        } else {
            self.root.insert(interval, aug_data, vec![data]);
        }
    }

    /// Returns a vector of non mutable references of all values belogning to intervals
    /// that cover *point*. The vector is ordered based on intervals' total order.
    ///
    /// #example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(0,10, "First Insertion");
    /// ist.insert(5, 20, "Second Insertion");
    /// assert_eq!(ist.at(2), [&"First Insertion"]);
    /// assert_eq!(ist.at(15), [&"Second Insertion"]);
    /// assert_eq!(ist.at(9), [&"First Insertion", &"Second Insertion"]);
    /// let empty_vector :Vec<&&str> = Vec::new();
    /// assert_eq!(ist.at(21), empty_vector);
    /// ```
    pub fn at(&self, point: K) -> Vec<&V> {
        if !self.root.is_node() {
            return Vec::new();
        }
        let accept = |key: &Interval<K>, point: &Interval<K>| key.has_point(point.lo);
        let recurse = |aug: &AugData<K>, point: &Interval<K>| aug.interval.has_point(point.lo);
        let point_int = Interval::new(point, point);
        self.root.generic_search(point_int, &recurse, &accept)
    }

    /// Returns a vector of mutable references of all values belogning to intervals
    /// that cover *point*. The vector is ordered based on intervals' total order.
    ///
    /// #example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(0,10, String::from("First Insertion"));
    /// ist.at_mut(5)[0].push_str(" Modified");
    /// assert_eq!(ist.at(5), [&"First Insertion Modified"]);
    /// ```
    pub fn at_mut(&mut self, point: K) -> Vec<&mut V> {
        if !self.root.is_node() {
            return Vec::new();
        }
        let accept = |key: &Interval<K>, point: &Interval<K>| key.has_point(point.lo);
        let recurse = |aug: &AugData<K>, point: &Interval<K>| aug.interval.has_point(point.lo);
        let point_int = Interval::new(point, point);
        self.root.generic_search_mut(point_int, &recurse, &accept)
    }

    /// Returns a vector of non mutable references of all values that belongs to intervals
    /// that envelop the interval specified by *[ lo, hi ]*. The vector is ordered based
    /// on intervals' total order.
    ///
    /// An interval *[ A, B ]* is said to be envloping interval
    /// *[ lo, hi ]* IFF *lo ≥ A* and *lo ≤ B* and *hi ≥ A* and *hi ≤ B*.
    /// # Panics
    /// Panics if *lo* > *hi*
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(0,10, "First Insertion");
    /// ist.insert(5, 20, "Second Insertion");
    /// assert_eq!(ist.envelop(8, 12), [&"Second Insertion"]);
    /// assert_eq!(ist.envelop(5, 10), [&"First Insertion", &"Second Insertion"]);
    /// assert_eq!(ist.envelop(0, 3), [&"First Insertion"]);
    /// let empty_vector :Vec<&&str> = Vec::new();
    /// assert_eq!(ist.envelop(0, 30), empty_vector);
    /// ```
    pub fn envelop(&self, lo: K, hi: K) -> Vec<&V> {
        assert!(lo <= hi);
        if !self.root.is_node() {
            return Vec::new();
        }
        let int = Interval::new(lo, hi);
        let accept = |big: &Interval<K>, small: &Interval<K>| big.envelop(small);
        let recruse = |big: &AugData<K>, small: &Interval<K>| big.interval.envelop(small);
        self.root.generic_search(int, &recruse, &accept)
    }

    /// Returns a vector of mutable references of all values that belongs to intervals
    /// that envelop the interval specified by *[ lo, hi ]*. The vector is ordered based
    /// on intervals' total order.
    ///
    /// An interval *[ A, B ]* is said to be envloping interval
    /// *[ lo, hi ]* IFF *lo ≥ A* and *lo ≤ B* and *hi ≥ A* and *hi ≤ B*.
    /// # Panics
    /// Panics if *lo* > *hi*
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(0,10, String::from("First Insertion"));
    /// ist.envelop_mut(7, 10)[0].push_str(" Modified");
    /// assert_eq!(ist.envelop(2, 4), [&"First Insertion Modified"]);
    /// ```
    pub fn envelop_mut(&mut self, lo: K, hi: K) -> Vec<&mut V> {
        assert!(lo <= hi);
        if !self.root.is_node() {
            return Vec::new();
        }
        let int = Interval::new(lo, hi);
        let accept = |big: &Interval<K>, small: &Interval<K>| big.envelop(small);
        let recurse = |big: &AugData<K>, small: &Interval<K>| big.interval.envelop(small);
        self.root.generic_search_mut(int, &recurse, &accept)
    }

    /// Returns a vector of non mutable references of all values that belongs to intervals
    /// that is enveloped interval specified by *[ lo, hi ]*. The vector is ordered based
    /// on intervals' total order.
    ///
    /// An interval *[ A, B ]* is said to be envloped by interval
    /// *[ lo, hi ]* IFF *lo ≤ A* and *lo ≤ B* and *hi ≥ A* and *hi ≥ B*.
    /// # Panics
    /// Panics if *lo* > *hi*
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(5,10, "First Insertion");
    /// ist.insert(15, 20, "Second Insertion");
    /// assert_eq!(ist.inverse_envelop(0, 12), [&"First Insertion"]);
    /// assert_eq!(ist.inverse_envelop(0, 20), [&"First Insertion", &"Second Insertion"]);
    /// assert_eq!(ist.inverse_envelop(12, 21), [&"Second Insertion"]);
    /// let empty_vector :Vec<&&str> = Vec::new();
    /// assert_eq!(ist.envelop(0, 7), empty_vector);
    /// ```
    pub fn inverse_envelop(&self, lo: K, hi: K) -> Vec<&V> {
        assert!(lo <= hi);
        if !self.root.is_node() {
            return Vec::new();
        }
        let int = Interval::new(lo, hi);
        let accept = |small: &Interval<K>, big: &Interval<K>| big.envelop(small);
        // There might be a better heurestic to tell if I should recurse left or right
        let recurse = |aug_data: &AugData<K>, int: &Interval<K>| aug_data.interval.overlap(int);
        self.root.generic_search(int, &recurse, &accept)
    }

    /// Returns a vector of non mutable references of all values that belongs to intervals
    /// that is enveloped interval specified by *[ lo, hi ]*. The vector is ordered based
    /// on intervals' total order.
    ///
    /// An interval *[ A, B ]* is said to be envloped by interval
    /// *[ lo, hi ]* IFF *lo ≤ A* and *lo ≤ B* and *hi ≥ A* and *hi ≥ B*.
    /// # Panics
    /// Panics if *lo* > *hi*
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(2,10, String::from("First Insertion"));
    /// ist.inverse_envelop_mut(0, 20)[0].push_str(" Modified");
    /// assert_eq!(ist.envelop(3, 5), [&"First Insertion Modified"]);
    /// ```
    pub fn inverse_envelop_mut(&mut self, lo: K, hi: K) -> Vec<&mut V> {
        assert!(lo <= hi);
        if !self.root.is_node() {
            return Vec::new();
        }
        let int = Interval::new(lo, hi);
        let accept = |small: &Interval<K>, big: &Interval<K>| big.envelop(small);
        // There might be a better heurestic to tell if I should recurse left or right
        let recurse = |aug_data: &AugData<K>, int: &Interval<K>| aug_data.interval.overlap(int);
        self.root.generic_search_mut(int, &recurse, &accept)
    }

    /// Returns a vector of non mutable references of all values that belongs to intervals
    /// that overlap with the interval specified by *[ lo, hi ]*. The vector is ordered based
    /// on intervals' total order.
    ///
    /// Two interval *[ A, B ]*, *[ lo, hi ]* are said to be overlapping IFF
    /// *max(A, lo) ≤ min(B, hi)*.
    /// # Panics
    /// Panics if *lo* > *hi*
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(0,20, "First Insertion");
    /// ist.insert(60, 80, "Second Insertion");
    /// assert_eq!(ist.overlap(40, 70), [&"Second Insertion"]);
    /// assert_eq!(ist.overlap(10, 40), [&"First Insertion"]);
    /// assert_eq!(ist.overlap(10, 100), [&"First Insertion", &"Second Insertion"]);
    /// let empty_vector :Vec<&&str> = Vec::new();
    /// assert_eq!(ist.envelop(30, 40), empty_vector);
    /// ```
    pub fn overlap(&self, lo: K, hi: K) -> Vec<&V> {
        assert!(lo <= hi);
        if !self.root.is_node() {
            return Vec::new();
        }
        let int = Interval::new(lo, hi);
        let accept = |int1: &Interval<K>, int2: &Interval<K>| int1.overlap(int2);
        let recurse = |aug_data: &AugData<K>, int: &Interval<K>| aug_data.interval.overlap(int);
        self.root.generic_search(int, &recurse, &accept)
    }
    /// Returns a vector of mutable references of all values that belongs to intervals that
    /// overlap with the interval specified by *[ lo, hi ]*. The vector is ordered based on
    /// intervals' total order.
    ///
    /// Two interval *[ A, B ]*, *[ lo, hi ]* are said to be overlapping IFF
    /// *max(A, lo) ≤ min(B, hi)*.
    /// # Panics
    /// Panics if *lo* > *hi*
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(10, 20, String::from("First Insertion"));
    /// ist.overlap_mut(7, 13)[0].push_str(" Modified");
    /// assert_eq!(ist.overlap(18, 25), [&"First Insertion Modified"]);
    /// ```
    pub fn overlap_mut(&mut self, lo: K, hi: K) -> Vec<&mut V> {
        assert!(lo <= hi);
        if !self.root.is_node() {
            return Vec::new();
        }
        let int = Interval::new(lo, hi);
        let accept = |int1: &Interval<K>, int2: &Interval<K>| int1.overlap(int2);
        let recurse = |aug_data: &AugData<K>, int: &Interval<K>| aug_data.interval.overlap(int);
        self.root.generic_search_mut(int, &recurse, &accept)
    }

    /// Deletes all Intervals that that cover *point*. The returned
    /// data is a vector of data stored inside the deleted intervals.
    ///
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(10, 30, String::from("First Insertion"));
    /// ist.insert(15, 35, String::from("Second Insertion"));
    /// assert_eq!(ist.at(25), [&"First Insertion", &"Second Insertion"]);
    /// assert_eq!(ist.delete_at(20), ["First Insertion", "Second Insertion"]);
    /// let empty_vec: Vec<&&'static str> = Vec::new();
    /// assert_eq!(ist.overlap(0, 50), empty_vec);
    /// ```
    pub fn delete_at(&mut self, point: K) -> Vec<V> {
        if !self.root.is_node() {
            return Vec::new();
        }
        let accept = |key: &Interval<K>, point: &Interval<K>| key.has_point(point.lo);
        let recurse = |aug: &AugData<K>, point: &Interval<K>| aug.interval.has_point(point.lo);
        let point_int = Interval::new(point, point);
        self.root.generic_delete(point_int, &recurse, &accept)
    }

    /// Deletes all Intervals that envelop the interval specified by *[ lo, hi ]*.
    /// The returned data is a vector of data stored inside the deleted intervals.
    ///
    /// An interval *[ A, B ]* is said to be envloping interval
    /// *[ lo, hi ]* IFF *lo ≥ A* and *lo ≤ B* and *hi ≥ A* and *hi ≤ B*.
    ///
    /// # Panics
    /// Panics if *lo* > *hi*
    ///
    /// # Example
    ///
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(10, 30, String::from("First Insertion"));
    /// ist.insert(15, 35, String::from("Second Insertion"));
    /// assert_eq!(ist.envelop(20, 25), [&"First Insertion", &"Second Insertion"]);
    /// assert_eq!(ist.delete_envelop(20, 25), ["First Insertion", "Second Insertion"]);
    /// let empty_vec: Vec<&&'static str> = Vec::new();
    /// assert_eq!(ist.envelop(20, 25), empty_vec);
    /// ```
    pub fn delete_envelop(&mut self, lo: K, hi: K) -> Vec<V> {
        assert!(lo <= hi);
        if !self.root.is_node() {
            return Vec::new();
        }
        let int = Interval::new(lo, hi);
        let accept = |big: &Interval<K>, small: &Interval<K>| big.envelop(small);
        let recurse = |big: &AugData<K>, small: &Interval<K>| big.interval.envelop(small);
        self.root.generic_delete(int, &recurse, &accept)
    }

    /// Deletes all Intervals that overlap with the interval specified by *[ lo, hi ]*.
    /// The returned data is a vector of data stored inside the deleted intervals.
    ///
    /// Two interval *[ A, B ]*, *[ lo, hi ]* are said to be overlapping IFF
    /// *max(A, lo) ≤ min(B, hi)*.
    /// # Panics
    /// Panics if *lo* > *hi*
    ///
    /// # Example
    /// ```
    /// use rair_trees::ist::IST;
    /// let mut ist = IST::new();
    /// ist.insert(5, 15, String::from("First Insertion"));
    /// ist.insert(10, 20, String::from("Second Insertion"));
    /// assert_eq!(ist.envelop(10, 15), [&"First Insertion", &"Second Insertion"]);
    /// assert_eq!(ist.delete_overlap(0, 7), ["First Insertion"]);
    /// assert_eq!(ist.delete_overlap(17, 27), ["Second Insertion"]);
    /// let empty_vec: Vec<&&'static str> = Vec::new();
    /// assert_eq!(ist.envelop(20, 25), empty_vec);
    /// ```

    pub fn delete_overlap(&mut self, lo: K, hi: K) -> Vec<V> {
        assert!(lo <= hi);
        if !self.root.is_node() {
            return Vec::new();
        }
        let int = Interval::new(lo, hi);
        let accept = |int1: &Interval<K>, int2: &Interval<K>| int1.overlap(int2);
        let recurse = |aug_data: &AugData<K>, int: &Interval<K>| aug_data.interval.overlap(int);

        self.root.generic_delete(int, &recurse, &accept)
    }
}

impl<K: Ord + Copy, V> IntoIterator for IST<K, V> {
    type Item = (K, K, V);
    type IntoIter = ISTIterator<K, V>;
    fn into_iter(self) -> ISTIterator<K, V> {
        ISTIterator::new(self)
    }
}

impl<'a, K: Ord + Copy, V> IntoIterator for &'a IST<K, V> {
    type Item = (K, K, &'a V);
    type IntoIter = ISTRefIterator<'a, K, V>;
    fn into_iter(self) -> ISTRefIterator<'a, K, V> {
        ISTRefIterator::new(self)
    }
}

#[cfg(test)]
mod ist_tests {
    use super::*;
    fn test_emptiness(ist: &mut IST<u64, &'static str>) {
        let empty_vec: Vec<&&'static str> = Vec::new();
        let empty_vec2: Vec<&'static str> = Vec::new();
        assert_eq!(ist.get_level(), 0);
        assert_eq!(ist.size(), 0);
        assert_eq!(ist.at(5), empty_vec);
        assert_eq!(ist.envelop(10, 20), empty_vec);
        assert_eq!(ist.overlap(0, 10), empty_vec);
        assert_eq!(ist.inverse_envelop(7, 21), empty_vec);
        assert_eq!(ist.at_mut(5), empty_vec);
        assert_eq!(ist.inverse_envelop_mut(7, 21), empty_vec);
        assert_eq!(ist.envelop_mut(10, 20), empty_vec);
        assert_eq!(ist.overlap_mut(0, 10), empty_vec);
        assert_eq!(ist.delete_envelop(10, 20), empty_vec2);
        assert_eq!(ist.delete_overlap(10, 20), empty_vec2);
        assert_eq!(ist.delete_at(10), empty_vec2);
    }
    #[test]
    fn test_empty_tree() {
        let mut ist = IST::new();
        test_emptiness(&mut ist);
    }

    fn get_a_good_tree() -> IST<u64, &'static str> {
        /*
         *  Range [0, 9] should always be empty
         *                                                  [50, 60]  max_hi = 200
         *                                                   /   \   min_lo = 10
         *                           _______________________/     \_______________________
         *                          /                                                     \
         *                      [20, 30] max_hi = 100                                  [80, 90] max_hi = 200
         *                       /   \   min_lo = 10                                    /   \   min_lo = 65
         *           ___________/     \____________                        ____________/     \____________
         *          /                              \                      /                               \
         *      [10, 100] max_hi = 100          [30, 40] max_hi = 40    [65, 70] max_hi = 200          [85, 95] max_hi = 95
         *                min_low = 10           /       min_lo = 25          \  min_lo = 65                    min_lo = 85
         *                                      /                              \
         *                                     /                                \
         *                               [25, 35] max_hi = 35                [66, 200] max_hi = 200
         *                                        min_lo = 25                          min_lo = 66
         */
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
        ist
    }

    #[test]
    fn test_1_node_tree() {
        let empty_vec: Vec<&&'static str> = Vec::new();
        let mut ist = IST::new();
        ist.insert(10, 30, "[10, 30]");
        assert_eq!(ist.get_level(), 1);
        assert_eq!(ist.size(), 1);
        assert_eq!(ist.at(17), [&"[10, 30]"]);
        assert_eq!(ist.at(10), [&"[10, 30]"]);
        assert_eq!(ist.at(30), [&"[10, 30]"]);
        assert_eq!(ist.at(31), empty_vec);
        assert_eq!(ist.at(9), empty_vec);
        assert_eq!(ist.envelop(15, 25), [&"[10, 30]"]);
        assert_eq!(ist.envelop(15, 30), [&"[10, 30]"]);
        assert_eq!(ist.envelop(10, 30), [&"[10, 30]"]);
        assert_eq!(ist.envelop(9, 30), empty_vec);
        assert_eq!(ist.envelop(10, 31), empty_vec);
        assert_eq!(ist.overlap(9, 31), [&"[10, 30]"]);
        assert_eq!(ist.overlap(9, 10), [&"[10, 30]"]);
        assert_eq!(ist.overlap(30, 50), [&"[10, 30]"]);
        assert_eq!(ist.overlap(1, 9), empty_vec);
        assert_eq!(ist.delete_envelop(10, 20), ["[10, 30]"]);
        test_emptiness(&mut ist);
    }

    #[test]
    fn test_insert() {
        let mut ist = IST::new();
        ist.insert(25, 30, "[25, 30]");
        assert_eq!(ist.get_level(), 1);
        assert_eq!(ist.size(), 1);
        ist.insert(20, 30, "[20, 30]");
        assert_eq!(ist.get_level(), 2);
        assert_eq!(ist.size(), 2);
        ist.insert(26, 30, "[26, 30]");
        assert_eq!(ist.get_level(), 2);
        assert_eq!(ist.size(), 3);
        ist.insert(10, 30, "[10, 30]");
        assert_eq!(ist.get_level(), 3);
        assert_eq!(ist.size(), 4);
    }
    #[test]
    fn test_n_nodes_size() {
        let ist = get_a_good_tree();
        assert_eq!(ist.size(), 9);
        assert_eq!(ist.get_level(), 4);
    }
    #[test]
    fn test_n_nodes_at() {
        let mut ist = get_a_good_tree();
        let empty_vec: Vec<&&'static str> = Vec::new();

        assert_eq!(ist.at(55), [&"[10, 100]", &"[50, 60]"]);
        assert_eq!(ist.at(100), [&"[10, 100]", &"[66, 200]"]);
        assert_eq!(ist.at(5), empty_vec);

        assert_eq!(ist.at_mut(55), [&"[10, 100]", &"[50, 60]"]);
        assert_eq!(ist.at_mut(100), [&"[10, 100]", &"[66, 200]"]);
        assert_eq!(ist.at_mut(5), empty_vec);
    }
    #[test]
    fn test_n_nodes_envelop() {
        let mut ist = get_a_good_tree();
        let empty_vec: Vec<&&'static str> = Vec::new();

        assert_eq!(ist.envelop(66, 200), [&"[66, 200]"]);
        assert_eq!(
            ist.envelop(66, 70),
            [&"[10, 100]", &"[65, 70]", &"[66, 200]"]
        );
        assert_eq!(ist.envelop(4, 9), empty_vec);

        assert_eq!(ist.envelop_mut(66, 200), [&"[66, 200]"]);
        assert_eq!(
            ist.envelop_mut(66, 70),
            [&"[10, 100]", &"[65, 70]", &"[66, 200]"]
        );
        assert_eq!(ist.envelop_mut(4, 9), empty_vec);
    }
    #[test]
    fn test_n_nodes_inverse_envelop() {
        let mut ist = get_a_good_tree();
        let empty_vec: Vec<&&'static str> = Vec::new();

        assert_eq!(ist.inverse_envelop(81, 100), [&"[85, 95]"]);
        assert_eq!(ist.inverse_envelop(81, 93), empty_vec);

        assert_eq!(ist.inverse_envelop_mut(81, 100), [&"[85, 95]"]);
        assert_eq!(ist.inverse_envelop_mut(81, 93), empty_vec);
    }
    #[test]
    fn test_n_nodes_overlap() {
        let mut ist = get_a_good_tree();
        let empty_vec: Vec<&&'static str> = Vec::new();

        assert_eq!(
            ist.overlap(62, 300),
            [
                &"[10, 100]",
                &"[65, 70]",
                &"[66, 200]",
                &"[80, 90]",
                &"[85, 95]"
            ]
        );
        assert_eq!(ist.overlap(4, 9), empty_vec);

        assert_eq!(
            ist.overlap_mut(62, 300),
            [
                &"[10, 100]",
                &"[65, 70]",
                &"[66, 200]",
                &"[80, 90]",
                &"[85, 95]"
            ]
        );
        assert_eq!(ist.overlap_mut(4, 9), empty_vec);
    }

    #[test]
    fn test_n_nodes_delete_envelop() {
        let mut ist = get_a_good_tree();
        let empty_vec: Vec<&&'static str> = Vec::new();
        assert_eq!(ist.delete_envelop(80, 200), ["[66, 200]"]);
        assert_eq!(ist.envelop(66, 70), [&"[10, 100]", &"[65, 70]"]);
        assert_eq!(ist.size(), 8);

        assert_eq!(ist.delete_envelop(25, 100), ["[10, 100]"]);
        assert_eq!(ist.envelop(66, 70), [&"[65, 70]"]);
        assert_eq!(ist.size(), 7);

        assert_eq!(ist.delete_envelop(30, 40), ["[30, 40]"]);
        assert_eq!(ist.envelop(30, 40), empty_vec);
        assert_eq!(ist.size(), 6);
        //Sanity check the tree
        assert_eq!(ist.envelop(20, 30), [&"[20, 30]"]);
        assert_eq!(ist.envelop(25, 35), [&"[25, 35]"]);
        assert_eq!(ist.envelop(50, 60), [&"[50, 60]"]);
        assert_eq!(ist.envelop(65, 70), [&"[65, 70]"]);
        assert_eq!(ist.envelop(80, 85), [&"[80, 90]"]);
        assert_eq!(ist.envelop(85, 95), [&"[85, 95]"]);

        //Tree is turning into a bush! lets get a new tree
        ist = get_a_good_tree();
        ist.delete_envelop(80, 200);
        ist.insert(66, 90, "[66, 90]");
        assert_eq!(ist.delete_envelop(65, 70), ["[10, 100]", "[65, 70]"]);
        ist.insert(68, 92, "[68, 92]");
        assert_eq!(ist.delete_envelop(50, 60), ["[50, 60]"]);
        assert_eq!(ist.envelop(50, 60), empty_vec);
        assert_eq!(ist.size(), 7);
        //Sanity check the tree
        assert_eq!(ist.envelop(20, 30), [&"[20, 30]"]);
        assert_eq!(ist.envelop(25, 35), [&"[25, 35]"]);
        assert_eq!(ist.envelop(66, 90), [&"[66, 90]"]);
        assert_eq!(ist.envelop(80, 90), [&"[66, 90]", &"[68, 92]", &"[80, 90]"]);
        assert_eq!(ist.envelop(68, 92), [&"[68, 92]"]);
        assert_eq!(ist.envelop(20, 30), [&"[20, 30]"]);
        assert_eq!(ist.envelop(85, 95), [&"[85, 95]"]);
        assert_eq!(ist.envelop(30, 40), [&"[30, 40]"]);
    }
    #[test]
    fn test_n_nodes_delete_at() {
        let mut ist = get_a_good_tree();
        assert_eq!(ist.delete_at(150), ["[66, 200]"]);
        assert_eq!(ist.size(), 8);
    }
    #[test]
    fn test_n_nodes_delete_overlap() {
        let mut ist = get_a_good_tree();
        assert_eq!(ist.delete_overlap(150, 210), ["[66, 200]"]);
        assert_eq!(ist.size(), 8);
    }
    #[test]
    fn test_dictionary_size() {
        // FIX #31
        let mut ist = get_a_good_tree();
        ist.insert(50, 60, "Attempt2");
        assert_eq!(ist.size(), 10)
    }
    #[test]
    fn test_iter() {
        let mut ist = get_a_good_tree();
        ist.insert(50, 60, "Attempt2");
        let mut iter = ist.into_iter();
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
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_ref() {
        let mut ist = get_a_good_tree();
        ist.insert(50, 60, "Attempt2");
        let mut iter = (&ist).into_iter();
        assert_eq!(iter.next().unwrap(), (10, 100, &"[10, 100]"));
        assert_eq!(iter.next().unwrap(), (20, 30, &"[20, 30]"));
        assert_eq!(iter.next().unwrap(), (25, 35, &"[25, 35]"));
        assert_eq!(iter.next().unwrap(), (30, 40, &"[30, 40]"));
        assert_eq!(iter.next().unwrap(), (50, 60, &"[50, 60]"));
        assert_eq!(iter.next().unwrap(), (50, 60, &"Attempt2"));
        assert_eq!(iter.next().unwrap(), (65, 70, &"[65, 70]"));
        assert_eq!(iter.next().unwrap(), (66, 200, &"[66, 200]"));
        assert_eq!(iter.next().unwrap(), (80, 90, &"[80, 90]"));
        assert_eq!(iter.next().unwrap(), (85, 95, &"[85, 95]"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        iter = (&ist).into_iter();
        assert_eq!(iter.next().unwrap(), (10, 100, &"[10, 100]"));
        assert_eq!(iter.next().unwrap(), (20, 30, &"[20, 30]"));
        assert_eq!(iter.next().unwrap(), (25, 35, &"[25, 35]"));
        assert_eq!(iter.next().unwrap(), (30, 40, &"[30, 40]"));
        assert_eq!(iter.next().unwrap(), (50, 60, &"[50, 60]"));
        assert_eq!(iter.next().unwrap(), (50, 60, &"Attempt2"));
        assert_eq!(iter.next().unwrap(), (65, 70, &"[65, 70]"));
        assert_eq!(iter.next().unwrap(), (66, 200, &"[66, 200]"));
        assert_eq!(iter.next().unwrap(), (80, 90, &"[80, 90]"));
        assert_eq!(iter.next().unwrap(), (85, 95, &"[85, 95]"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}
