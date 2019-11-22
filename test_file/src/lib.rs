/*
 * file_test.rs: Library for aiding unit testing Rair IO.
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

extern crate tempfile;

use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

pub const DATA: &[u8] = &[
    0x00, 0x01, 0x01, 0x02, 0x03, 0x05, 0x08, 0x0d, 0x15, 0x22, 0x37, 0x59, 0x90, 0xe9, 0x79, 0x62, 0xdb, 0x3d, 0x18, 0x55, 0x6d, 0xc2, 0x2f, 0xf1, 0x20, 0x11, 0x31, 0x42, 0x73, 0xb5, 0x28, 0xdd,
    0x05, 0xe2, 0xe7, 0xc9, 0xb0, 0x79, 0x29, 0xa2, 0xcb, 0x6d, 0x38, 0xa5, 0xdd, 0x82, 0x5f, 0xe1, 0x40, 0x21, 0x61, 0x82, 0xe3, 0x65, 0x48, 0xad, 0xf5, 0xa2, 0x97, 0x39, 0xd0, 0x09, 0xd9, 0xe2,
    0xbb, 0x9d, 0x58, 0xf5, 0x4d, 0x42, 0x8f, 0xd1, 0x60, 0x31, 0x91, 0xc2, 0x53, 0x15, 0x68, 0x7d, 0xe5, 0x62, 0x47, 0xa9, 0xf0, 0x99, 0x89, 0x22, 0xab, 0xcd, 0x78, 0x45, 0xbd, 0x02, 0xbf, 0xc1,
    0x80, 0x41, 0xc1, 0x02, 0xc3, 0xc5, 0x88, 0x4d, 0xd5,
];

pub fn operate_on_file(test_function: &dyn Fn(&Path), data: &[u8]) {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(data).unwrap();
    test_function(file.path());
}

pub fn operate_on_copy(test_function: &dyn Fn(&Path), path: &str) {
    let mut file = NamedTempFile::new().unwrap();
    //file.write_all(data).unwrap();
    fs::copy(path, &mut file).unwrap();
    test_function(file.path());
}

pub fn operate_on_files(test_function: &dyn Fn(&[&Path]), files_data: &[&[u8]]) {
    let mut files: Vec<NamedTempFile> = Vec::with_capacity(files_data.len());
    let mut paths: Vec<&Path> = Vec::with_capacity(files_data.len());
    // This suck! the 2 loops thing, but rust compiler didn't let me do otherwise.
    for i in 0..files_data.len() {
        files.push(NamedTempFile::new().unwrap());
        files[i].write_all(files_data[i]).unwrap();
    }
    for file in &files {
        paths.push(file.path());
    }
    test_function(&paths);
}
