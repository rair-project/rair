/*
 *  {one line to give the program's name and a brief idea of what it does.}
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
//TODO delete the whole file ^_^
use libc::*;
#[link(name = "r_io")]
extern {
    pub fn r_io_new() -> *mut c_void;
    pub fn r_io_size(io: *mut c_void) -> usize;
    pub fn r_io_open_nomap(io: *mut c_void, file: *const c_char, flags :c_int, mode: c_int) -> *const c_void;
    pub fn r_io_pwrite(io: *mut c_void, paddr:u64, buf: *const u8, len:usize);
    pub fn r_io_pread(io: *mut c_void, paddr:u64, buff: *const u8, len:usize);
}
