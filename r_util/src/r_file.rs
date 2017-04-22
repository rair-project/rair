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
//TODO delete the whole file ^_^
extern crate libc;
use libc::*;
use std::ffi::CString;
#[link(name = "r_util")]
extern "C" {
    fn r_file_is_directory(file: *const c_char) -> bool;
}
pub fn is_directory(file: &str) -> bool {
    let cstring = CString::new(file).unwrap();
    unsafe { r_file_is_directory(cstring.as_ptr()) }
}
