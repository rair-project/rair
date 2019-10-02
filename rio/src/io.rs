/**
 * io.rs: RIO main implementation.
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
 **/
use defaultplugin;
use desc::*;
use plugin::*;
use std::cmp::min;
use std::io;
use utils::*;
pub struct RIO {
    descs: Vec<RIODesc>, // sorted vector based on RIODesc.paddr
    maps: Vec<RIOMap>,   // sorted vector based on RIOMap.
    plugins: Vec<Box<dyn RIOPlugin>>,
    default_plugin: Box<dyn RIOPlugin>,
}

pub struct RIOMap {
    paddr: u64,
    vaddr: u64,
    size: u64,
}

impl RIO {
    fn get_new_hndl(&self) -> Result<u64, IoError> {
        for i in 0..u64::max_value() {
            let desc = self.descs.iter().find(|&d| i == d.get_hndl());
            if desc.is_none() {
                return Ok(i);
            }
        }
        return Err(IoError::TooManyFilesError);
    }

    fn insert_desc_at(&mut self, mut desc: RIODesc, paddr: u64) -> Result<u64, IoError> {
        let insert_before_me = self.descs.iter().position(|d| paddr + desc.size as u64 <= d.paddr);
        let location: usize;
        match insert_before_me {
            Some(i) => {
                if i == 0 {
                    location = i;
                } else {
                    let prevdesc = &self.descs[i - 1];
                    if prevdesc.has_paddr(paddr) {
                        return Err(IoError::AddressesOverlapError);
                    }
                    location = i;
                }
            }
            _ => {
                location = self.descs.len() - 1;
            }
        }
        desc.paddr = paddr;
        self.descs.insert(location, desc);
        return Ok(self.descs[location].get_hndl());
    }

    fn insert_desc(&mut self, mut desc: RIODesc) -> &mut RIODesc {
        if self.descs.len() == 0 {
            self.descs.push(desc);
            return &mut self.descs[0];
        }
        let mut where_to_insert: usize = 0;
        let mut paddr: u64 = 0;
        for i in 0..self.descs.len() {
            if i + 1 < self.descs.len() {
                let tmp_paddr = self.descs[i].paddr + self.descs[i].size;
                if tmp_paddr + desc.size <= self.descs[i + 1].paddr {
                    paddr = tmp_paddr;
                    where_to_insert = i + 1;
                }
            } else {
                paddr = self.descs[i].paddr + self.descs[i].size;
                where_to_insert = i + 1;
                break;
            }
        }
        desc.paddr = paddr;
        self.descs.insert(where_to_insert, desc);
        return &mut self.descs[where_to_insert];
    }

    fn try_open(&mut self, uri: &str, flags: IoMode) -> Result<RIODesc, IoError> {
        let hndl = self.get_new_hndl()?;
        for plugin in &mut self.plugins {
            if plugin.accept_uri(uri) {
                return RIODesc::open(plugin, uri, flags, hndl);
            }
        }
        if self.default_plugin.accept_uri(uri) {
            return RIODesc::open(&mut self.default_plugin, uri, flags, hndl);
        }
        return Err(IoError::IoPluginNotFoundError);
    }

    pub fn new() -> RIO {
        let ret: RIO = RIO {
            descs: Vec::new(),
            maps: Vec::new(),
            plugins: Vec::new(),
            default_plugin: defaultplugin::plugin(),
        };
        return ret;
    }

    pub fn load_plugin(&mut self, plugin: Box<dyn RIOPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn open(&mut self, uri: &str, flags: IoMode) -> Result<u64, IoError> {
        let desc = self.try_open(uri, flags)?;
        let desc_ref = self.insert_desc(desc);
        return Ok(desc_ref.get_hndl());
    }

    pub fn open_at(&mut self, uri: &str, flags: IoMode, at: u64) -> Result<u64, IoError> {
        let desc = self.try_open(uri, flags)?;
        return self.insert_desc_at(desc, at);
    }

    pub fn close(&mut self, hndl: u64) {
        //delete all memory mappings related to the closed handle
        self.descs.retain(|desc| desc.get_hndl() != hndl);
    }

    pub fn close_all(&mut self) {
        self.maps.clear();
        self.descs.clear();
    }

    pub fn pread(&mut self, paddr: u64, buf: &mut [u8]) -> Result<(), IoError> {
        let result = self.descs.iter().position(|x| paddr >= x.paddr && paddr < x.paddr + x.size);
        match result {
            Some(mut i) => {
                // the orders variable here represents the read operation orders that needs to be later fulfilled;
                //collect all the i, offset from paddr, size
                let mut orders: Vec<(usize, usize, usize)> = Vec::new();
                let mut offset = 0;
                while offset != buf.len() {
                    if i >= self.descs.len() {
                        return Err(IoError::Parse(io::Error::new(io::ErrorKind::UnexpectedEof, "BufferOverflow")));
                    }
                    let size = min(buf.len() - offset, self.descs[i].size as usize);
                    orders.push((i, offset, size));
                    offset += size;
                    i += 1;
                }
                for (i, delta, size) in orders {
                    self.descs[i].read(paddr as usize + delta, &mut buf[delta..delta + size])?;
                }
                return Ok(());
            }
            None => return Err(IoError::AddressNotFound),
        }
    }

    pub fn pwrite(&mut self, paddr: u64, buf: &[u8]) -> Result<(), IoError> {
        let result = self.descs.iter().position(|x| paddr >= x.paddr && paddr < x.paddr + x.size);
        match result {
            Some(mut i) => {
                // the orders variable here represents the read operation orders that needs to be later fulfilled;
                //collect all the i, offset from paddr, size
                let mut orders: Vec<(usize, usize, usize)> = Vec::new();
                let mut offset = 0;
                while offset != buf.len() {
                    if i >= self.descs.len() {
                        return Err(IoError::Parse(io::Error::new(io::ErrorKind::UnexpectedEof, "BufferOverflow")));
                    }
                    let size = min(buf.len() - offset, self.descs[i].size as usize);
                    orders.push((i, offset, size));
                    offset += size;
                    i += 1;
                }
                for (i, delta, size) in orders {
                    self.descs[i].write(paddr as usize + delta, &buf[delta..delta + size])?;
                }
                return Ok(());
            }
            None => return Err(IoError::AddressNotFound),
        }
    }

    pub fn phy_to_hndl(&self, paddr: u64) -> Option<u64> {
        let desc = self.descs.iter().find(|x| paddr >= x.paddr && paddr < x.paddr + x.size)?;
        return Some(desc.get_hndl());
    }

    pub fn is_phy(&self, paddr: u64, size: u64) -> bool {
        let result = self.descs.iter().position(|x| paddr >= x.paddr && paddr < x.paddr + x.size);
        match result {
            Some(mut i) => {
                let mut offset = 0;
                while offset != size {
                    if i >= self.descs.len() {
                        return false;
                    }
                    let delta = min(size - offset, self.descs[i].size);
                    offset += delta;
                    i += 1;
                }
                return true;
            }
            None => return false,
        }
    }
    pub fn map(&mut self, paddr: u64, vaddr: u64, size: u64) -> Result<(), IoError> {
        // check if paddr till paddr + size is valid
        if !self.is_phy(paddr, size) {
            return Err(IoError::AddressNotFound);
        }
        // check if vaddr is previosly used or not
        let insert_before_me = self.maps.iter().position(|x| vaddr + size < x.vaddr);
        let location: usize;
        match insert_before_me {
            Some(i) => {
                if i == 0 {
                    location = i;
                } else {
                    let prevmap = &self.maps[i - 1];
                    if vaddr >= prevmap.vaddr && vaddr < prevmap.vaddr + prevmap.size {
                        return Err(IoError::AddressesOverlapError);
                    }
                    location = i;
                }
            }
            _ => {
                location = self.maps.len() - 1;
            }
        }
        // do the mapping
        let map = RIOMap {
            paddr: paddr,
            vaddr: vaddr,
            size: size,
        };
        self.maps.insert(location, map);
        return Ok(());
    }
    fn split_maps(&mut self, i: usize, vaddr: u64) {
        let delta = self.maps[i].vaddr - vaddr;
        let new_map = RIOMap {
            vaddr: vaddr,
            paddr: self.maps[i].paddr + delta,
            size: self.maps[i].size - delta,
        };
        self.maps[i].size = delta;
        self.maps.insert(i + 1, new_map);
    }
    pub fn unmap(&mut self, vaddr: u64, size: u64) {
        let unmap_here = self.maps.iter().position(|x| vaddr >= x.vaddr && vaddr <= x.vaddr + x.size);
        if let Some(mut i) = unmap_here {
            let mut progress = 0;
            while progress != size {
                if i >= self.maps.len() {
                    break;
                }
                // if the start address is in the middle of the map first split the map
                if vaddr > self.maps[i].vaddr {
                    self.split_maps(i, vaddr);
                    i += 1
                }
                // if the end address is at the middle of the map first split the map
                if vaddr + size < self.maps[i].vaddr + self.maps[i].size {
                    self.split_maps(i, vaddr + size);
                }
                progress += self.maps[i].size;
                self.maps.remove(i);
            }
        }
    }

    pub fn vread(&mut self, vaddr: u64, buf: &mut [u8]) -> Result<(), IoError> {
        let result = self.maps.iter().position(|x| vaddr >= x.vaddr && vaddr < x.vaddr + x.size);
        match result {
            Some(mut i) => {
                // the orders variable here represents the read operation orders that needs to be later fulfilled;
                //collect all the paddr, size
                let mut orders: Vec<(usize, usize)> = Vec::new();
                let mut offset = 0;
                while offset != buf.len() {
                    if i >= self.maps.len() {
                        return Err(IoError::Parse(io::Error::new(io::ErrorKind::UnexpectedEof, "BufferOverflow")));
                    }
                    let size = min(buf.len() - offset, self.maps[i].size as usize);
                    let paddr = (vaddr - self.maps[i].vaddr + self.maps[i].paddr) as usize + offset;
                    orders.push((paddr, size));
                    offset += size;
                    i += 1;
                }
                offset = 0;
                for (paddr, size) in orders {
                    self.pread(paddr as u64, &mut buf[offset..size])?;
                    offset += size;
                }
                return Ok(());
            }
            None => return Err(IoError::AddressNotFound),
        }
    }
    pub fn vwrite(&mut self, vaddr: u64, buf: &[u8]) -> Result<(), IoError> {
        let result = self.maps.iter().position(|x| vaddr >= x.vaddr && vaddr < x.vaddr + x.size);
        match result {
            Some(mut i) => {
                // the orders variable here represents the read operation orders that needs to be later fulfilled;
                //collect all the paddr, size
                let mut orders: Vec<(usize, usize)> = Vec::new();
                let mut offset = 0;
                while offset != buf.len() {
                    if i >= self.maps.len() {
                        return Err(IoError::Parse(io::Error::new(io::ErrorKind::UnexpectedEof, "BufferOverflow")));
                    }
                    let size = min(buf.len() - offset, self.maps[i].size as usize);
                    let paddr = (vaddr - self.maps[i].vaddr + self.maps[i].paddr) as usize + offset;
                    orders.push((paddr, size));
                    offset += size;
                    i += 1;
                }
                offset = 0;
                for (paddr, size) in orders {
                    self.pwrite(paddr as u64, &buf[offset..size])?;
                    offset += size;
                }
                return Ok(());
            }
            None => return Err(IoError::AddressNotFound),
        }
    }
    pub fn vir_to_phy(&self, vaddr: u64) -> Option<u64> {
        let i = self.maps.iter().position(|x| vaddr >= x.vaddr && vaddr < x.vaddr + x.size)?;
        return Some(vaddr - self.maps[i].vaddr + self.maps[i].paddr);
    }
    pub fn is_vir(&self, vaddr: u64, size: u64) -> bool {
        let result = self.maps.iter().position(|x| vaddr >= x.vaddr && vaddr < x.vaddr + x.size);
        match result {
            Some(mut i) => {
                let mut offset = 0;
                while offset != size {
                    if i >= self.maps.len() {
                        return false;
                    }
                    let delta = min(size - offset, self.maps[i].size);
                    offset += delta;
                    i += 1;
                }
                return true;
            }
            None => return false,
        }
    }
}

#[cfg(test)]
mod rio_tests {
    use super::*;
    use std::path::Path;
    use test_aids::*;
    fn test_open_close_cb(path: &Path) {
        let mut io = RIO::new();
        let hndl = io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        assert_eq!(io.descs.len(), 1);
        assert_eq!(hndl, 0);
        io.close(hndl);
        assert_eq!(io.descs.len(), 0);
    }
    #[test]
    fn test_open_close() {
        operate_on_file(&test_open_close_cb, DATA);
    }
    // TODO: failing open, failing reads, and failing writes.
    fn test_phy_to_hndl_cb(paths: &[&Path]) {
        let mut io = RIO::new();
        io.open(&paths[0].to_string_lossy(), IoMode::READ).unwrap();
        io.open_at(&paths[1].to_string_lossy(), IoMode::READ, 0x2000).unwrap();
        io.open_at(&paths[2].to_string_lossy(), IoMode::READ, 0x1000).unwrap();
        assert_eq!(io.phy_to_hndl(0x10).unwrap(), 0);
        assert_eq!(io.phy_to_hndl(0x2000).unwrap(), 1);
        assert_eq!(io.phy_to_hndl(0x1000).unwrap(), 2);
        assert_eq!(io.phy_to_hndl(0x500), None);
    }
    #[test]
    fn test_phy_to_hndl() {
        operate_on_files(&test_phy_to_hndl_cb, &[DATA, DATA, DATA]);
    }
    fn test_pread_cb(paths: &[&Path]) {
        let mut io = RIO::new();
        let mut fillme: Vec<u8> = vec![0; 8];

        for path in paths {
            io.open(&path.to_string_lossy(), IoMode::READ).unwrap();
        }
        // First normal read
        io.pread(0, &mut fillme).unwrap();
        assert_eq!(fillme, &DATA[0..8]);
        // Second we read through 1 desc into another desc
        fillme = vec![0; DATA.len() * 3 / 2];
        io.pread(0, &mut fillme).unwrap();
        let mut sanity_data: Vec<u8> = vec![0; (DATA.len() * 3 / 2)];
        sanity_data[0..DATA.len()].copy_from_slice(DATA);
        let mut l = sanity_data.len() - DATA.len();
        sanity_data[DATA.len()..DATA.len() * 3 / 2].copy_from_slice(&DATA[0..l]);
        assert_eq!(fillme, sanity_data);
        // Now we make sure that we can read through all three descs
        fillme = vec![0; DATA.len() * 5 / 2];
        io.pread(0, &mut fillme).unwrap();
        sanity_data = vec![0; (DATA.len() * 5 / 2)];
        sanity_data[0..DATA.len()].copy_from_slice(DATA);
        sanity_data[DATA.len()..DATA.len() * 2].copy_from_slice(DATA);
        let mut l = sanity_data.len() - DATA.len() * 2;
        sanity_data[DATA.len() * 2..DATA.len() * 5 / 2].copy_from_slice(&DATA[0..l]);
        assert_eq!(fillme, sanity_data);
    }
    #[test]
    fn test_pread() {
        operate_on_files(&test_pread_cb, &[DATA, DATA, DATA]);
    }

    fn test_pwrite_cb(paths: &[&Path]) {
        let mut io = RIO::new();
        let mut fillme: Vec<u8> = vec![0; 8];

        for path in paths {
            io.open(&path.to_string_lossy(), IoMode::READ | IoMode::WRITE).unwrap();
        }
        // First normal write
        io.pwrite(0, &fillme).unwrap();
        io.pread(0, &mut fillme).unwrap();
        assert_eq!(fillme, &[0; 8]);
        // Second we write through 1 desc into another desc
        fillme = vec![1; DATA.len() * 3 / 2];
        io.pwrite(0, &fillme).unwrap();
        io.pread(0, &mut fillme).unwrap();
        assert_eq!(fillme, vec![1; DATA.len() * 3 / 2]);
        // Now we make sure that we can write through all three descs
        fillme = vec![2; DATA.len() * 5 / 2];
        io.pwrite(0, &fillme).unwrap();
        io.pread(0, &mut fillme).unwrap();
        assert_eq!(fillme, vec![2; DATA.len() * 5 / 2]);
    }
    #[test]
    fn test_pwrite() {
        operate_on_files(&test_pwrite_cb, &[DATA, DATA, DATA]);
    }
    /*
        #[test]
        fn test_phy_vir() {
            assert!(false, "Not tested yet!");
            let io = io_new().unwrap();
            io_open_at(io, "malloc://512", IoMode::READ, 0x1000);
            let addr = io_vir2phy(io, 0x1100);
            io_close(io);
            io_free(io);
        }
    */
}
