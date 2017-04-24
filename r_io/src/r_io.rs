/*
 *  Copyright (C) 2017  Ahmed Abd El Mawgood
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use libc::*;
use std::ffi::CString;
pub const READ: i32 = 4;
#[link(name = "r_io")]
extern "C" {
    fn r_io_new() -> *const c_void;
    fn r_io_size(io: *const c_void) -> usize;
    fn r_io_open_nomap(io: *const c_void,
                       file: *const c_char,
                       flags: c_int,
                       mode: c_int)
                       -> *const c_void;
    fn r_io_pwrite(io: *const c_void, paddr: u64, buf: *const u8, len: usize);
    fn r_io_pread(io: *const c_void, paddr: u64, buff: *const u8, len: usize);
    fn r_io_seek(io: *const c_void, offset: u64, whence: i32);
}
pub fn new<'a>() -> &'a c_void {
    unsafe { &*r_io_new() }
}
pub enum Seek {
    Set,
    Cur,
    End,
}
pub fn size(io: &c_void) -> usize {
    unsafe { r_io_size(io) }
}
pub fn open_nomap(io: &c_void, file: &str, flags: i32, mode: i32) -> *const c_void {
    let cstr = CString::new(file).unwrap();
    unsafe { r_io_open_nomap(io, cstr.as_ptr(), flags, mode) }
}
pub fn pwrite(io: &c_void, paddr: u64, buf: &[u8]) {
    unsafe { r_io_pwrite(io, paddr, buf.as_ptr(), buf.len()) };
}
pub fn pread(io: &c_void, paddr: u64, buf: &mut [u8]) {
    unsafe { r_io_pread(io, paddr, buf.as_ptr(), buf.len()) };
}
pub fn seek(io: &c_void, offset: u64, whence: Seek) {
    unsafe { r_io_seek(io, offset, whence as i32) };
}
