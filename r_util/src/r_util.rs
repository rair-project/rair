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
extern crate libc;
use libc::c_char;
#[link(name = "r_util")]
extern {
    pub fn r_num_math (RNUM: *const c_char, STR: *const c_char) -> u64;
    pub fn r_file_is_directory (file: *const c_char) -> bool;
    pub fn r_print_progressbar (RPrint: *const c_char, pc: i32, cols:i32);
    pub fn r_print_randomart (dgst_raw: *const u8, dgst_raw_len:usize, addr:usize) -> *const c_char;
}
