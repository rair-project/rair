/*
 * mapsquery.rs: Data structure that enables queries and reverse queries on vaddr <--> paddr.
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
use rtrees::ist::IST;
use std::cmp::min;
use std::rc::Rc;
use utils::*;
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct RIOMap {
    pub paddr: u64,
    pub vaddr: u64,
    pub size: u64,
}

impl RIOMap {
    fn has_paddr(&self, paddr: u64) -> bool {
        paddr >= self.paddr && paddr < self.paddr + self.size
    }

    fn has_vaddr(&self, vaddr: u64) -> bool {
        vaddr >= self.vaddr && vaddr < self.vaddr + self.size
    }

    fn envelop(&self, map: &RIOMap) -> bool {
        self.has_paddr(map.paddr) && self.has_paddr(map.paddr + map.size - 1) && self.has_vaddr(map.vaddr) && self.has_vaddr(map.vaddr + map.size - 1)
    }
    fn split(mut self, vaddr: u64) -> (RIOMap, RIOMap) {
        let delta = vaddr - self.vaddr;
        let new_map = RIOMap {
            vaddr,
            paddr: self.paddr + delta,
            size: self.size - delta,
        };
        self.size = delta;
        return (self, new_map);
    }
    // This will only work IFF self.envelop(map) == true
    fn remove_projection(mut self, map: &RIOMap) -> Vec<RIOMap> {
        let mut maps = Vec::with_capacity(2);
        if self.vaddr < map.vaddr {
            let (clean, tainted) = self.split(map.vaddr);
            maps.push(clean);
            self = tainted;
        }
        if map.vaddr + map.size < self.vaddr + self.size {
            let (_, clean) = self.split(map.vaddr + map.size);
            maps.push(clean);
        }
        return maps;
    }
}

impl PartialEq<RIOMap> for Rc<RIOMap> {
    fn eq(&self, other: &RIOMap) -> bool {
        return &**self == other;
    }
}

impl PartialEq<Rc<RIOMap>> for RIOMap {
    fn eq(&self, other: &Rc<RIOMap>) -> bool {
        return self == &**other;
    }
}

#[derive(Default)]
pub(super) struct RIOMapQuery {
    maps: IST<u64, Rc<RIOMap>>,     //key = virtual address
    rev_maps: IST<u64, Rc<RIOMap>>, // key = physiscal address
}

impl RIOMapQuery {
    pub fn new() -> RIOMapQuery {
        RIOMapQuery {
            maps: IST::new(),
            rev_maps: IST::new(),
        }
    }
    pub fn map(&mut self, paddr: u64, vaddr: u64, size: u64) -> Result<(), IoError> {
        // check if vaddr is previosly used or not
        if !self.maps.overlap(vaddr, vaddr + size - 1).is_empty() {
            return Err(IoError::AddressesOverlapError);
        }
        let mapping = Rc::new(RIOMap { paddr, vaddr, size });
        self.maps.insert(vaddr, vaddr + size - 1, mapping.clone());
        self.rev_maps.insert(paddr, paddr + size - 1, mapping);
        return Ok(());
    }
    pub fn split_vaddr_range(&self, vaddr: u64, size: u64) -> Option<Vec<RIOMap>> {
        let maps: Vec<Rc<RIOMap>> = self.maps.overlap(vaddr, vaddr + size - 1).iter().map(|&x| x.clone()).collect();
        if maps.is_empty() {
            return None;
        }
        let mut ranged_hndl = Vec::with_capacity(maps.len());
        let mut start = vaddr;
        let mut remaining = size;
        for map in maps {
            if start < map.vaddr {
                return None;
            }
            let delta = min(remaining, map.size - (start - map.vaddr));
            let frag = RIOMap {
                paddr: map.paddr + (start - map.vaddr),
                vaddr: start,
                size: delta,
            };
            ranged_hndl.push(frag);
            start += delta;
            remaining -= delta;
        }
        if remaining != 0 {
            return None;
        }
        return Some(ranged_hndl);
    }
    pub fn split_vaddr_sparce_range(&self, vaddr: u64, size: u64) -> Vec<RIOMap> {
        let maps: Vec<Rc<RIOMap>> = self.maps.overlap(vaddr, vaddr + size - 1).iter().map(|&x| x.clone()).collect();
        if maps.is_empty() {
            return Vec::new();
        }
        let mut ranged_hndl = Vec::with_capacity(maps.len());
        let mut start = vaddr;
        let mut remaining = size;
        for map in maps {
            if start < map.vaddr {
                remaining -= map.vaddr - start;
                start = map.vaddr;
            }
            let delta = min(remaining, map.size - (start - map.vaddr));
            let frag = RIOMap {
                paddr: map.paddr + (start - map.vaddr),
                vaddr: start,
                size: delta,
            };
            ranged_hndl.push(frag);
            start += delta;
            remaining -= delta;
        }
        return ranged_hndl;
    }
    pub fn unmap(&mut self, vaddr: u64, size: u64) -> Result<(), IoError> {
        let fragments = self.split_vaddr_range(vaddr, size);
        if fragments.is_none() {
            return Err(IoError::AddressNotFound);
        }
        for frag in fragments.unwrap() {
            let old_map = self.maps.delete_envelop(frag.vaddr, frag.vaddr + frag.size - 1)[0].clone();
            let old_rev_maps = self.rev_maps.delete_envelop(frag.paddr, frag.paddr + frag.size - 1);
            // we will get 1 normal map and maybe many rev_maps,
            // The reason is that 1 vaddr can only point to 1 paddr
            // but 1 paddr can be pointed to by many vaddrs
            old_map
                .remove_projection(&frag)
                .into_iter()
                .map(|m| m)
                .for_each(|m| self.maps.insert(m.vaddr, m.vaddr + m.size - 1, Rc::new(m)));
            for map in old_rev_maps {
                if map.envelop(&frag) {
                    map.remove_projection(&frag)
                        .into_iter()
                        .map(|m| m)
                        .for_each(|m| self.rev_maps.insert(m.paddr, m.paddr + m.size - 1, Rc::new(m)));
                } else {
                    self.rev_maps.insert(map.paddr, map.paddr + map.size - 1, map)
                }
            }
        }
        return Ok(());
    }
}

impl<'a> IntoIterator for &'a RIOMapQuery {
    type Item = Rc<RIOMap>;
    type IntoIter = Box<dyn Iterator<Item = Rc<RIOMap>> + 'a>;
    fn into_iter(self) -> Box<dyn Iterator<Item = Rc<RIOMap>> + 'a> {
        return Box::new((&self.maps).into_iter().cloned());
    }
}

#[cfg(test)]
mod maps_query_test {
    use super::*;
    #[test]
    fn test_map_unmap() {
        let mut map_query = RIOMapQuery::new();

        // simple file open, map and unmap
        map_query.map(0, 0x4000, 0x100).unwrap();
        assert_eq!(map_query.maps.size(), 1);
        assert_eq!(map_query.rev_maps.size(), 1);

        map_query.unmap(0x4000, 0x100).unwrap();
        assert_eq!(map_query.maps.size(), 0);
        assert_eq!(map_query.rev_maps.size(), 0);

        map_query.map(0, 0x4000, 0x100).unwrap();
        map_query.map(0x100, 0x5000, 0x100).unwrap();
        map_query.map(0x200, 0x2000, 0x100).unwrap();
        map_query.map(0x300, 0x3000, 0x100).unwrap();
        assert_eq!(map_query.maps.size(), 4);
        assert_eq!(map_query.rev_maps.size(), 4);

        map_query.unmap(0x5000, 0x100).unwrap();
        map_query.unmap(0x2000, 0x100).unwrap();
        map_query.unmap(0x3000, 0x100).unwrap();
        map_query.unmap(0x4000, 0x100).unwrap();
        assert_eq!(map_query.maps.size(), 0);
        assert_eq!(map_query.rev_maps.size(), 0);

        map_query.map(0, 0x1000, 0x300).unwrap();
        assert_eq!(map_query.maps.size(), 1);

        map_query.unmap(0x1100, 0x100).unwrap();
        assert_eq!(map_query.maps.size(), 2);

        assert_eq!(map_query.split_vaddr_range(0x1000, 0x100).unwrap(), vec![RIOMap { vaddr: 0x1000, paddr: 0, size: 0x100 }]);
        assert_eq!(
            map_query.split_vaddr_range(0x1200, 0x100).unwrap(),
            vec![RIOMap {
                vaddr: 0x1200,
                paddr: 0x200,
                size: 0x100
            }]
        );
        assert_eq!(map_query.split_vaddr_range(0x1100, 0x100), None);
    }

    #[test]
    fn test_map_errors() {
        let mut map_query = RIOMapQuery::new();
        map_query.map(0x1000, 0x4000, 0x1000).unwrap();
        let mut e = map_query.map(0x3000, 0x4100, 0x1000).err();
        assert_eq!(e.unwrap(), IoError::AddressesOverlapError);
        assert_eq!(map_query.split_vaddr_range(0x3000, 0x2000), None);
        assert_eq!(map_query.split_vaddr_range(0x3000, 0x3000), None);
        e = map_query.unmap(0x3500, 0x500).err();
        assert_eq!(e.unwrap(), IoError::AddressNotFound);
    }

    #[test]
    fn test_map_iter() {
        let mut map_query = RIOMapQuery::new();
        map_query.map(0, 0x4000, 0x100).unwrap();
        map_query.map(0x100, 0x5000, 0x100).unwrap();
        map_query.map(0x200, 0x2000, 0x100).unwrap();
        map_query.map(0x300, 0x3000, 0x100).unwrap();
        let mut iter = map_query.into_iter();
        assert_eq!(
            RIOMap {
                paddr: 0x200,
                vaddr: 0x2000,
                size: 0x100
            },
            iter.next().unwrap()
        );
        assert_eq!(
            RIOMap {
                paddr: 0x300,
                vaddr: 0x3000,
                size: 0x100
            },
            iter.next().unwrap()
        );
        assert_eq!(RIOMap { paddr: 0, vaddr: 0x4000, size: 0x100 }, iter.next().unwrap());
        assert_eq!(
            RIOMap {
                paddr: 0x100,
                vaddr: 0x5000,
                size: 0x100
            },
            iter.next().unwrap()
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_split_vaddr_sparce_range() {
        let mut map_query = RIOMapQuery::new();
        map_query.map(0, 0x4000, 0x90).unwrap();
        map_query.map(0x100, 0x5000, 0x90).unwrap();
        map_query.map(0x200, 0x2000, 0x90).unwrap();
        map_query.map(0x300, 0x3000, 0x90).unwrap();
        assert_eq!(
            map_query.split_vaddr_sparce_range(0x1000, 0x5000),
            vec![
                RIOMap {
                    paddr: 0x200,
                    vaddr: 0x2000,
                    size: 0x90
                },
                RIOMap {
                    paddr: 0x300,
                    vaddr: 0x3000,
                    size: 0x90
                },
                RIOMap {
                    paddr: 0x0,
                    vaddr: 0x4000,
                    size: 0x90
                },
                RIOMap {
                    paddr: 0x100,
                    vaddr: 0x5000,
                    size: 0x90
                }
            ]
        );
    }
}
